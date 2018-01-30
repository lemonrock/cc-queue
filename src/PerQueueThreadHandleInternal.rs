// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


// TODO: DOUBLE_CACHE_ALIGNED
#[derive(Debug)]
#[repr(C)]
struct PerQueueThreadHandleInternal<T, A: Allocator>
{
	enq: SynchHandle<T>,
	deq: SynchHandle<T>,
	
	// Used for object pooling; recycles dequeue'd Node<T> to avoid additional calls to `allocate_next_node()`.
	// Can be null
	// If not null, is **never** fully initialized.
	next: *mut Node<T>,
	
	allocator: A,
}

impl<T, A: Allocator> Drop for PerQueueThreadHandleInternal<T, A>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		if self.next.is_not_null()
		{
			self.allocator.free_cache_line_size(unsafe { NonNull::new_unchecked(self.next) });
		}
	}
}

impl<T, A: Allocator> PerQueueThreadHandleInternal<T, A>
{
	#[inline(always)]
	fn allocate_next_node(&mut self) -> NonNull<Node<T>>
	{
		Self::allocate_next_node_(&mut self.allocator)
	}
	
	#[inline(always)]
	fn allocate_next_node_(allocator: &mut A) -> NonNull<Node<T>>
	{
		allocator.align_malloc_cache_line_size()
	}
	
	#[inline(always)]
	fn free_after_drop(this: NonNull<Self>)
	{
		HeapAllocator.free_page_size(this)
	}
	
	// happens once per-thread
	#[inline(always)]
	fn new(mut allocator: A) -> NonNull<Self>
	{
		let mut handle = HeapAllocator.align_malloc_page_size();
		unsafe
		{
			let handle: &mut Self = handle.as_mut();
			
			handle.enq.ccsynch_handle_init();
			handle.deq.ccsynch_handle_init();
			
			write(&mut handle.next, Self::allocate_next_node_(&mut allocator).as_ptr());
			
			write(&mut handle.allocator, allocator);
		}
		handle
	}
}
