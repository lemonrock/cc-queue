// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


/// This structure is allocated for each thread that wants to access a queue.
#[derive(Debug)]
pub struct PerQueueThreadHandle<'queue, T: 'queue, A: 'queue + Allocator>(&'queue CcQueue<T, A>, NonNull<PerQueueThreadHandleInternal<T, A>>);

impl<'queue, T, A: Allocator> Drop for PerQueueThreadHandle<'queue, T, A>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let pointer = self.1;
		unsafe { drop_in_place(pointer.as_ptr()) }
		PerQueueThreadHandleInternal::free_after_drop(pointer);
	}
}

impl<'queue, T, A: Allocator> PerQueueThreadHandle<'queue, T, A>
{
	/// Enqueue data.
	#[inline(always)]
	pub fn enqueue(&mut self, data: NonNull<T>)
	{
		let queue = unsafe { (self.0).0.as_ref() };
		
		queue.enqueue(self.handle(), data)
	}
	
	/// Dequeue data.
	#[inline(always)]
	pub fn dequeue(&mut self) -> Option<NonNull<T>>
	{
		let queue = unsafe { (self.0).0.as_ref() };
		
		queue.dequeue(self.handle())
	}
	
	#[inline(always)]
	fn handle(&mut self) -> &mut PerQueueThreadHandleInternal<T, A>
	{
		unsafe { (self.1).as_mut() }
	}
}
