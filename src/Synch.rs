// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct Synch<T>
{
	// Never null
	tail: AtomicPtr<SynchNode<T>>, // TODO: Make 128-byte aligned
}

impl<T> Drop for Synch<T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let tail = self.tail.load(Acquire);
		unsafe { drop_in_place(tail) };
		SynchNode::free_after_drop(unsafe { NonNull::new_unchecked(tail) })
	}
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
