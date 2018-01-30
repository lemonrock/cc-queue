// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


/// This trait is for objects that need to reset state when an allocator is opened.
/// Typically this is for objects that store temporary state to persistent memory.
pub trait AllocatorOpened<A: Allocator>
{
	#[inline(always)]
	fn allocator_opened(&mut self, allocator: A);
}
