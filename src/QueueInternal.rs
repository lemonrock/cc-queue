// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


// TODO: DOUBLE_CACHE_ALIGNED
#[derive(Debug)]
#[repr(C)]
struct QueueInternal<T, A: Allocator>
{
	enq: UnsafeCell<Synch<T>>, // TODO: DOUBLE_CACHE_ALIGNED
	deq: UnsafeCell<Synch<T>>, // TODO: DOUBLE_CACHE_ALIGNED
	head: UnsafeCell<NonNull<Node<T>>>, // TODO: DOUBLE_CACHE_ALIGNED
	tail: UnsafeCell<NonNull<Node<T>>>, // TODO: DOUBLE_CACHE_ALIGNED
	allocator: UnsafeCell<A>,
}

impl<T, A: Allocator> AllocatorOpened<A> for QueueInternal<T, A>
{
	#[inline(always)]
	fn allocator_opened(&mut self, allocator: A)
	{
		unsafe
		{
			Synch::ccsynch_init(&self.enq);
			Synch::ccsynch_init(&self.deq);
		
			write(&mut self.allocator, UnsafeCell::new(allocator))
		}
	}
}

impl<T, A: Allocator> Drop for QueueInternal<T, A>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		#[inline(always)]
		fn drop_node<T, A: Allocator>(node: &UnsafeCell<NonNull<Node<T>>>, allocator: &mut A)
		{
			let node = unsafe { *node.get() };
			Node::clearing_queue_drop(node, allocator)
		}
		
		let allocator = self.allocator();
		
		// head and tail can be the same (see `new` below)!
		if self.head.get() != self.tail.get()
		{
			drop_node(&self.head, allocator);
			drop_node(&self.tail, allocator);
		}
		else
		{
			drop_node(&self.head, allocator);
		}
	}
}

impl<T, A: Allocator> QueueInternal<T, A>
{
	#[inline(always)]
	fn allocator(&self) -> &mut A
	{
		unsafe { &mut *self.allocator.get() }
	}
	
	#[inline(always)]
	fn free_after_drop(this: NonNull<Self>, mut allocator: A)
	{
		allocator.free_page_size(this)
	}
	
	#[inline(always)]
	fn new(mut allocator: A) -> NonNull<Self>
	{
		let mut queue = allocator.align_malloc_page_size();
		
		unsafe
		{
			let queue: &mut Self = queue.as_mut();
			
			Synch::ccsynch_init(&queue.enq);
			Synch::ccsynch_init(&queue.deq);
			
			let dummy = Node::dummy_node(&mut allocator);
			
			write(&mut queue.head, UnsafeCell::new(dummy));
			write(&mut queue.tail, UnsafeCell::new(dummy));
			
			write(&mut queue.allocator, UnsafeCell::new(allocator))
		}
		
		queue
	}
	
	// handle is a per-thread object
	fn enqueue(&self, handle: &mut PerQueueThreadHandleInternal<T, A>, data: NonNull<T>)
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
			// Object pooling
			let mut node = if node.is_not_null()
			{
				write(&mut handle.next, null_mut());
				NonNull::new_unchecked(node)
			}
			else
			{
				handle.allocate_next_node()
			};
			
			let node = node.as_mut();
			write(&mut node.data, data);
			write(&mut node.next, null_mut());
			Self::ccsynch_apply(&self.enq, &mut handle.enq, serial_enqueue, &self.tail, node)
		}
	}
	
	// handle is a per-thread object
	fn dequeue(&self, handle: &mut PerQueueThreadHandleInternal<T, A>) -> Option<NonNull<T>>
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
					
					// Object pooling
					if handle.next.is_not_null()
					{
						Node::free_after_drop(node, self.allocator())
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
	unsafe fn ccsynch_apply<D, Apply: Fn(&UnsafeCell<NonNull<Node<T>>>, &mut D)>(synch: &UnsafeCell<Synch<T>>, synch_handle: &mut SynchHandle<T>, apply: Apply, state: &UnsafeCell<NonNull<Node<T>>>, data: &mut D)
	{
		let mut next = synch_handle.next;
		
		{
			let next = next.as_mut();
			write(&mut next.next, AtomicPtr::new(null_mut()));
			write(&mut next.status, AtomicU32::new(Status::WAIT as u32));
		}
		
		let mut current = Synch::swap_tail_returning_previous(synch, next);
		write(&mut synch_handle.next, current);
		
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
