// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct SynchNode<T>
{
	next: AtomicPtr<SynchNode<T>>, // TODO: Make 64-byte cache-line aligned
	data: *mut (),
	status: AtomicU32, // TODO: Make 64-byte cache-line aligned
}

impl<T> Drop for SynchNode<T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let next = self.acquire_next();
		if next.is_not_null()
		{
			unsafe { drop_in_place(next) };
			Self::free_after_drop(unsafe { NonNull::new_unchecked(next) })
		}
	}
}

impl<T> SynchNode<T>
{
	#[inline(always)]
	fn free_after_drop(this: NonNull<Self>)
	{
		HeapAllocator.free_cache_line_size(this)
	}
	
	#[inline(always)]
	unsafe fn ccsynch_init_node() -> NonNull<Self>
	{
		let mut node = HeapAllocator.align_malloc_cache_line_size();
		{
			let node: &mut Self = node.as_mut();
			
			write(&mut node.next, AtomicPtr::new(null_mut()));
			
			// Not strictly required as will be overwritten always.
			write(&mut node.data, null_mut());
			
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
