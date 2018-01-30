// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


use ::std::heap::Alloc;
use ::std::heap::Heap;
use ::std::heap::Layout;
use ::std::mem::size_of;
use ::std::ptr::NonNull;


include!("Allocator.rs");
include!("AllocatorOpened.rs");
include!("HeapAllocator.rs");
