use pool::pool;

// for radio packets we'll use these blocks as:
// { padding: 3B, len: 1B, data: 127B, LQI: 1B }
//
// and for USB packets we'll use them:
// { padding: 4B, data: 64B, padding: 63B }
//
// this let's us convert between them with zero copies
//
// the padding is needed because USB.data must be 4-byte aligned
pool!(pub P: [u8; 132]);
