// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


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
	fn clearing_queue_drop<A: Allocator>(this: NonNull<Self>, allocator: &mut A)
	{
		let x = unsafe { this.as_ref() };
		if x.next.is_not_null()
		{
			Self::clearing_queue_drop(unsafe { NonNull::new_unchecked(x.next) }, allocator);
		}
		Self::free_after_drop(this, allocator)
		// FIXME: free this.data !!!
	}
	
	#[inline(always)]
	fn free_after_drop<A: Allocator>(this: NonNull<Self>, allocator: &mut A)
	{
		allocator.free_cache_line_size(this)
	}
	
	#[inline(always)]
	unsafe fn dummy_node<A: Allocator>(allocator: &mut A) -> NonNull<Self>
	{
		let mut dummy = allocator.align_malloc_cache_line_size();
		{
			let dummy: &mut Self = dummy.as_mut();
			// This data is believed to be always overwritten...
			write(&mut dummy.data, NonNull::dangling());
			write(&mut dummy.next, null_mut());
		}
		dummy
	}
}
