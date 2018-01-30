// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct SynchHandle<T>
{
	next: NonNull<SynchNode<T>>,
}

impl<T> Drop for SynchHandle<T>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let next = self.next;
		unsafe { drop_in_place(next.as_ptr()) };
		SynchNode::free_after_drop(next)
	}
}

impl<T> SynchHandle<T>
{
	#[inline(always)]
	unsafe fn ccsynch_handle_init(&mut self)
	{
		write(&mut self.next, SynchNode::ccsynch_init_node())
	}
}
