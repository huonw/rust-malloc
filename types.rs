pub struct Header {
    prev: Box,
    next: Box,
    size: uint,
    free: bool
}

impl Header {
    pub fn default() -> Header {
        Header {
            prev: Box::null(),
            next: Box::null(),
            size: 0,
            free: true
        }
    }
}

#[inline(always)]
pub fn header_size() -> uint {
    // function rather than macro because a macro complains about
    // unnecessary `unsafe` if used inside an `unsafe` block
    unsafe { ::zero::size_of::<Header>() }
}

// (header, data...)
pub struct Box(*mut Header);
// [header](data...)
pub struct Data(*mut u8);

impl Box {
    #[inline(always)]
    pub fn null() -> Box { Box::from_uint(0) }
    #[inline(always)]
    pub fn is_null(&self) -> bool { **self as uint == 0 }

    #[inline(always)]
    pub fn from_ptr(ptr: *mut u8) -> Box { Box(ptr as *mut Header) }
    #[inline(always)]
    pub fn from_uint(u: uint) -> Box { Box(u as *mut Header) }

    #[inline]
    pub fn data(&self) -> Data {
        Data::from_uint((**self as uint) + header_size())
    }

    /// Returns the box that fits immediately after this allocation.
    #[inline]
    pub fn next_box_by_size(&self) -> Box {
        // 3-star programming!
        Box::from_uint((**self) as uint + header_size() + self.size())
    }

    /// Splits a box into a non-free and free section, updating the
    /// linked list appropriately. This doesn't check that the
    /// `data_size` argument is sensible (i.e. can fit within the self
    /// box).
    pub fn split_box(&self, data_size: uint) -> Box {
        let old_size = self.size();

        assert!(old_size != 0 || !self.has_next(),
                "Calling split_box on an empty within the list");
        assert!(old_size == 0 || data_size + header_size() < old_size,
                "Calling split_box without enough space");

        self.header().size = data_size;

        let new_box = self.next_box_by_size();

        // update the back links
        if self.has_next() {
            self.next().header().prev = new_box;
        }

        *new_box.header() = Header {
            prev: *self,
            next: self.next(),
            // If the current box is 0 sized and we're calling split,
            // then we're probably at the end, so the trailing one
            // should also be 0 sized. If the current box has space,
            // then subtract off size of data, and size of this header
            // to work out how much remains.
            size: if old_size == 0 {0} else {old_size - data_size - header_size()},
            free: true
        };

        self.header().next = new_box;
        self.header().free = false;

        new_box
    }

    #[inline]
    pub fn next(&self) -> Box {
        self.header().next
    }
    #[inline]
    pub fn has_next(&self) -> bool {
        !self.next().is_null()
    }

    #[inline]
    pub fn prev(&self) -> Box {
        self.header().prev
    }
    #[inline]
    pub fn has_prev(&self) -> bool {
        !self.prev().is_null()
    }
    #[inline]
    pub fn size(&self) -> uint {
        self.header().size
    }
    #[inline]
    pub fn is_free(&self) -> bool {
        self.header().free
    }

    #[inline(always)]
    pub fn header<'a>(&'a self) -> &'a mut Header {
        assert!(!self.is_null(), "header of a null Box")
        unsafe { &mut ***self }
    }


    /// Attempt to merge self and self.next, only succeeds if they are
    /// both non-null and both free.
    pub fn try_merge(&self) {
        if self.is_null() { return; }
        let n = self.next();
        if self.is_free() &&
            !n.is_null() && n.is_free() {

            let nn = n.next();
            self.header().next = nn;
            if !nn.is_null() {
                nn.header().prev = *self;
            }

            self.header().size += header_size() + n.size();
        }
    }
}

impl Data {
    #[inline(always)]
    pub fn from_uint(u: uint) -> Data { Data(u as *mut u8) }
    #[inline(always)]
    pub fn box(&self) -> Box {
        Box::from_uint((**self as uint) - header_size())
    }
}