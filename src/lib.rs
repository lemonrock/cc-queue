// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of cc-queue, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


#![feature(integer_atomics)]


use ::std::cell::UnsafeCell;
use ::std::mem::size_of;
use ::std::mem::transmute;
use ::std::mem::transmute_copy;
use ::std::ptr::NonNull;
use ::std::ptr::null_mut;
use ::std::ptr::write;
use ::std::sync::atomic::AtomicPtr;
use ::std::sync::atomic::AtomicU32;
use ::std::sync::atomic::Ordering::AcqRel;
use ::std::sync::atomic::Ordering::Acquire;
use ::std::sync::atomic::Ordering::Release;
use ::std::sync::atomic::spin_loop_hint as PAUSE;


include!("IsNotNull.rs");


#[derive(Debug, Copy, Clone)]
pub struct Queue<T>(NonNull<QueueInternal<T>>);

unsafe impl<T> Send for Queue<T>
{
}
unsafe impl<T> Sync for Queue<T>
{
}

impl<T> Queue<T>
{
	/// Create a new queue.
	#[inline(always)]
	pub fn new() -> Self
	{
		Queue(QueueInternal::new())
	}
	
	/// Create a new per-thread handle
	#[inline(always)]
	pub fn new_per_thread_handle<'queue>(&'queue self) -> PerQueueThreadHandle<'queue, T>
	{
		PerQueueThreadHandle(self, PerQueueThreadHandleInternal::new())
	}
}

#[derive(Debug)]
pub struct PerQueueThreadHandle<'queue, T: 'queue>(&'queue Queue<T>, NonNull<PerQueueThreadHandleInternal<T>>);

impl<'queue, T> Drop for PerQueueThreadHandle<'queue, T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		if self.handle().next.is_not_null()
		{
			// TODO: Use the Heap allocator for this?
			free(self.handle().next)
		}
		free(self)
	}
}

impl<'queue, T> PerQueueThreadHandle<'queue, T>
{
	#[inline(always)]
	pub fn enqueue(&mut self, data: NonNull<T>)
	{
		let queue = unsafe { (self.0).0.as_ref() };
		
		queue.enqueue(self.handle(), data)
	}
	
	#[inline(always)]
	pub fn dequeue(&mut self) -> Option<NonNull<T>>
	{
		let queue = unsafe { (self.0).0.as_ref() };
		
		queue.dequeue(self.handle())
	}
	
	#[inline(always)]
	fn handle(&mut self) -> &mut PerQueueThreadHandleInternal<T>
	{
		unsafe { (self.1).as_mut() }
	}
}

// TODO: DOUBLE_CACHE_ALIGNED
#[derive(Debug)]
#[repr(C)]
struct QueueInternal<T>
{
	enq: UnsafeCell<Synch<T>>, // TODO: DOUBLE_CACHE_ALIGNED
	deq: UnsafeCell<Synch<T>>, // TODO: DOUBLE_CACHE_ALIGNED
	head: UnsafeCell<NonNull<Node<T>>>, // TODO: DOUBLE_CACHE_ALIGNED
	tail: UnsafeCell<NonNull<Node<T>>>, // TODO: DOUBLE_CACHE_ALIGNED
}

impl<T> QueueInternal<T>
{
	#[inline(always)]
	fn new() -> NonNull<Self>
	{
		let mut queue = align_malloc(PAGE_SIZE);
		
		unsafe
		{
			let queue: &mut Self = queue.as_mut();
			
			Synch::ccsynch_init(&queue.enq);
			Synch::ccsynch_init(&queue.deq);
			
			let dummy = Node::dummy_node();
			
			write(&mut queue.head, UnsafeCell::new(dummy));
			write(&mut queue.tail, UnsafeCell::new(dummy));
		}
		
		queue
	}
	
	// handle is a per-thread object
	fn enqueue(&self, handle: &mut PerQueueThreadHandleInternal<T>, data: NonNull<T>)
	{
		#[inline(always)]
		fn serial_enqueue<T>(tail: &UnsafeCell<NonNull<Node<T>>>, node: &mut Node<T>)
		{
			let tail = tail.get();
			
			unsafe
			{
				// (*tail)->next = node
				write(&mut (*tail).as_mut().next, node);
				
				// *tail = node
				write(tail, NonNull::new_unchecked(node))
			}
		}
		
		let node = handle.next;
		
		unsafe
		{
			let mut node = if node.is_not_null()
			{
				write(&mut handle.next, null_mut());
				NonNull::new_unchecked(node)
			}
			else
			{
				 align_malloc(CACHE_LINE_SIZE)
			};
			
			let node = node.as_mut();
			write(&mut node.data, data);
			write(&mut node.next, null_mut());
			Self::ccsynch_apply(&self.enq, &mut handle.enq, serial_enqueue, &self.tail, node)
		}
	}
	
	// handle is a per-thread object
	fn dequeue(&self, handle: &mut PerQueueThreadHandleInternal<T>) -> Option<NonNull<T>>
	{
		#[inline(always)]
		fn serial_dequeue<T>(head: &UnsafeCell<NonNull<Node<T>>>, result: &mut Option<NonNull<Node<T>>>)
		{
			let head = head.get();
			
			let mut node = unsafe { *head };
			
			let next = unsafe { node.as_ref() }.next;
			if next.is_not_null()
			{
				let next = unsafe { NonNull::new_unchecked(next) };
				unsafe
				{
					write(&mut node.as_mut().data, next.as_ref().data);
					*head = next;
				}
				
				*result = Some(node)
			}
			else
			{
				*result = None
			}
		}
		
		let mut node: Option<NonNull<Node<T>>> = None;
		
		unsafe
		{
			Self::ccsynch_apply(&self.deq, &mut handle.deq, serial_dequeue, &self.head, &mut node);
			
			match node
			{
				None => None,
				Some(node) =>
				{
					let data = node.as_ref().data;
					
					if handle.next.is_not_null()
					{
						free(node.as_ptr())
					}
					else
					{
						write(&mut handle.next, node.as_ptr())
					}
					
					Some(data)
				}
			}
		}
	}
	
	#[inline(always)]
	unsafe fn ccsynch_apply<D, Apply: Fn(&UnsafeCell<NonNull<Node<T>>>, &mut D)>(synch: &UnsafeCell<Synch<T>>, handle: &mut SynchHandle<T>, apply: Apply, state: &UnsafeCell<NonNull<Node<T>>>, data: &mut D)
	{
		let mut next = handle.next;
		
		{
			let next = next.as_mut();
			write(&mut next.next, AtomicPtr::new(null_mut()));
			write(&mut next.status, AtomicU32::new(Status::WAIT as u32));
		}
		
		let mut current = Synch::swap_tail_returning_previous(synch, next);
		write(&mut handle.next, current);
		
		let mut status = current.as_ref().acquire_status();
		
		if status == Status::WAIT
		{
			write(&mut current.as_mut().data, transmute_copy(data));
			current.as_mut().release_next(next);
			
			// a do-while loop
			while
			{
				PAUSE();
				status = current.as_ref().acquire_status();
				status == Status::WAIT
			}
			{
			}
		}
		
		if status != Status::DONE
		{
			apply(state, data);
			
			let mut current = next;
			
			// next can be null
			let mut next = current.as_ref().acquire_next();
			
			let mut count: usize = 0;
			const CCSYNCH_HELP_BOUND: usize = 256;
			while next.is_not_null() && count < CCSYNCH_HELP_BOUND
			{
				apply(state, transmute_copy(&current.as_ref().data));
				current.as_mut().release_status_done();
				
				current = NonNull::new_unchecked(next);
				
				// next can be null
				next = current.as_ref().acquire_next();
				
				count += 1;
			}
			
			current.as_mut().release_status_ready();
		}
	}
}


// TODO: DOUBLE_CACHE_ALIGNED
/// There is one of these objects per thread.
#[derive(Debug)]
#[repr(C)]
struct PerQueueThreadHandleInternal<T>
{
	enq: SynchHandle<T>,
	deq: SynchHandle<T>,
	
	// Can be null
	next: *mut Node<T>,
}

impl<T> PerQueueThreadHandleInternal<T>
{
	// happens once per-thread
	/*
	void thread_init(int id, int nprocs) {
 	 hds[id] = align_malloc(PAGE_SIZE, sizeof(handle_t));
  		queue_register(q, hds[id], id);
	}
	*/
	#[inline(always)]
	fn new() -> NonNull<Self>
	{
		// TODO: Use the Heap allocator for this?
		let mut handle = align_malloc(PAGE_SIZE);
		unsafe
		{
			let handle: &mut Self = handle.as_mut();
			
			handle.enq.ccsynch_handle_init();
			handle.deq.ccsynch_handle_init();
			
			write(&mut handle.next, align_malloc(CACHE_LINE_SIZE).as_mut());
		}
		handle
	}
}

#[derive(Debug)]
#[repr(C)]
struct SynchHandle<T>
{
	next: NonNull<SynchNode<T>>,
}

impl<T> SynchHandle<T>
{
	#[inline(always)]
	unsafe fn ccsynch_handle_init(&mut self)
	{
		write(&mut self.next, align_malloc(CACHE_LINE_SIZE))
	}
}

#[derive(Debug)]
#[repr(C)]
struct Synch<T>
{
	// Never null
	tail: AtomicPtr<SynchNode<T>>, // TODO: Make 128-byte aligned
}

impl<T> Synch<T>
{
	#[inline(always)]
	unsafe fn ccsynch_init(this: &UnsafeCell<Synch<T>>)
	{
		let this = { &mut * this.get() };
		this.tail = AtomicPtr::new(SynchNode::ccsynch_init_node().as_ptr());
	}
	
	#[inline(always)]
	fn swap_tail_returning_previous(this: &UnsafeCell<Synch<T>>, next: NonNull<SynchNode<T>>) -> NonNull<SynchNode<T>>
	{
		let this = unsafe { &* this.get() };
		let raw = this.tail.swap(next.as_ptr(), AcqRel);
		unsafe { NonNull::new_unchecked(raw) }
	}
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(u32)]
enum Status
{
	WAIT = 0x0,
	READY = 0x1,
	DONE = 0x3,
}

#[derive(Debug)]
#[repr(C)]
struct Node<T>
{
	next: *mut Node<T>, // TODO: CACHE_ALIGNED
	data: NonNull<T>, // except the dummy node's data can be null
}

impl<T> Node<T>
{
	#[inline(always)]
	unsafe fn dummy_node() -> NonNull<Self>
	{
		let mut dummy = align_malloc(CACHE_LINE_SIZE);
		{
			let dummy: &mut Self = dummy.as_mut();
			// This data is believed to be always overwritten...
			write(&mut dummy.data, NonNull::dangling());
			write(&mut dummy.next, null_mut());
		}
		dummy
	}
}

#[derive(Debug)]
#[repr(C)]
struct SynchNode<T>
{
	next: AtomicPtr<SynchNode<T>>, // TODO: Make 64-byte cache-line aligned
	data: *mut (),
	status: AtomicU32, // TODO: Make 64-byte cache-line aligned
}

impl<T> SynchNode<T>
{
	#[inline(always)]
	unsafe fn ccsynch_init_node() -> NonNull<Self>
	{
		let mut node = align_malloc(CACHE_LINE_SIZE);
		{
			let node: &mut Self = node.as_mut();
			
			write(&mut node.next, AtomicPtr::new(null_mut()));
			
			write(&mut node.status, AtomicU32::new(Status::READY as u32));
		}
		node
	}
	
	// Result can be null
	#[inline(always)]
	fn acquire_next(&self) -> *mut SynchNode<T>
	{
		self.next.load(Acquire)
	}
	
	#[inline(always)]
	fn release_next(&mut self, next: NonNull<SynchNode<T>>)
	{
		self.next.store(next.as_ptr(), Release)
	}
	
	#[inline(always)]
	fn acquire_status(&self) -> Status
	{
		unsafe { transmute(self.status.load(Acquire)) }
	}
	
	#[inline(always)]
	fn release_status_done(&mut self)
	{
		self.release_status(Status::DONE);
	}
	
	#[inline(always)]
	fn release_status_ready(&mut self)
	{
		self.release_status(Status::READY);
	}
	
	#[inline(always)]
	fn release_status(&mut self, status: Status)
	{
		self.status.store(status as u32, Release);
	}
}

const PAGE_SIZE: usize = 4096;

const CACHE_LINE_SIZE: usize = 64;

#[inline(always)]
fn align_malloc<P>(_alignment: usize) -> NonNull<P>
{
	let _size = size_of::<P>();
	
	unimplemented!();
}

#[inline(always)]
fn free<P>(_pointer: *mut P)
{
	unimplemented!();
}
