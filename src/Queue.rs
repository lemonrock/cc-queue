// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


/// This is the cc queue object.
/// It is safe to send references between threads.
/// Each thread accessing the queue should call `new_per_thread_handle`.
#[derive(Debug)]
pub struct Queue<T, A: Allocator>(NonNull<QueueInternal<T, A>>);

unsafe impl<T, A: Allocator> Send for Queue<T, A>
{
}

unsafe impl<T, A: Allocator> Sync for Queue<T, A>
{
}

impl<T, A: Allocator> AllocatorOpened<A> for Queue<T, A>
{
	#[inline(always)]
	fn allocator_opened(&mut self, allocator: A)
	{
		unsafe { self.0.as_mut() }.allocator_opened(allocator)
	}
}

impl<T, A: Allocator> Drop for Queue<T, A>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let queue_internal = self.0;
		let allocator = unsafe { self.0.as_ref() }.allocator().clone();
		unsafe { drop_in_place(queue_internal.as_ptr()) };
		QueueInternal::free_after_drop(queue_internal, allocator);
	}
}

impl<T, A: Allocator> Queue<T, A>
{
	/// Create a new queue.
	#[inline(always)]
	pub fn new(allocator: A) -> Self
	{
		Queue(QueueInternal::new(allocator))
	}
	
	/// Create a new per-thread handle.
	#[inline(always)]
	pub fn new_per_thread_handle<'queue>(&'queue self) -> PerQueueThreadHandle<'queue, T, A>
	{
		PerQueueThreadHandle(self, PerQueueThreadHandleInternal::new(unsafe { self.0.as_ref() }.allocator().clone()))
	}
}
