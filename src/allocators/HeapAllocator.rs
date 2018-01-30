// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


/// A heap allocator allocates and frees memory from the heap; this is the default Rust allocator.
#[derive(Debug, Copy, Clone)]
pub struct HeapAllocator;

impl Allocator for HeapAllocator
{
	#[inline(always)]
	fn align_malloc<P>(&mut self, alignment: usize) -> NonNull<P>
	{
		let size = size_of::<P>();
		
		let mut heap = Heap;
		
		unsafe { NonNull::new_unchecked(heap.alloc(Layout::from_size_align_unchecked(size, alignment)).unwrap() as *mut P) }
	}
	
	#[inline(always)]
	fn free_page_size<P>(&mut self, pointer: NonNull<P>)
	{
		let size = size_of::<P>();
		
		let mut heap = Heap;
		
		unsafe { heap.dealloc(pointer.as_ptr() as *mut u8, Layout::from_size_align_unchecked(size, Self::PAGE_SIZE)) }
	}
	
	#[inline(always)]
	fn free_cache_line_size<P>(&mut self, pointer: NonNull<P>)
	{
		let size = size_of::<P>();
		
		let mut heap = Heap;
		
		unsafe { heap.dealloc(pointer.as_ptr() as *mut u8, Layout::from_size_align_unchecked(size, Self::CACHE_LINE_SIZE)) }
	}
	
	#[inline(always)]
	fn free<P>(&mut self, pointer: NonNull<P>)
	{
		let mut heap = Heap;
		
		unsafe { heap.dealloc(pointer.as_ptr() as *mut u8, Layout::new::<P>()) }
	}
}
