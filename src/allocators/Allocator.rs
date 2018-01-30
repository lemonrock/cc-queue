// This file is part of cc-queue. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of cc-queue. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/cc-queue/master/COPYRIGHT.


/// An allocator allocates and frees memory.
/// Allocators must implement a simple clone that returns an object that refers to the same memory pool.
pub trait Allocator: Clone
{
	const PAGE_SIZE: usize = 4096;
	
	const CACHE_LINE_SIZE: usize = 64;
	
	/// allocators memory like alloc, but aligned on page size.
	#[inline(always)]
	fn align_malloc_page_size<P>(&mut self) -> NonNull<P>
	{
		self.align_malloc(Self::PAGE_SIZE)
	}
	
	/// allocators memory like alloc, but aligned on cache line.
	#[inline(always)]
	fn align_malloc_cache_line_size<P>(&mut self) -> NonNull<P>
	{
		self.align_malloc(Self::CACHE_LINE_SIZE)
	}
	
	/// allocators memory like alloc, but aligned.
	#[inline(always)]
	fn align_malloc<P>(&mut self, alignment: usize) -> NonNull<P>;
	
	/// frees previously allocated memory that was aligned on page size.
	#[inline(always)]
	fn free_page_size<P>(&mut self, pointer: NonNull<P>);
	
	/// frees previously allocated memory that was aligned on cache line size.
	#[inline(always)]
	fn free_cache_line_size<P>(&mut self, pointer: NonNull<P>);
	
	/// frees previously allocated memory.
	#[inline(always)]
	fn free<P>(&mut self, pointer: NonNull<P>);
}
