// tmg1_c.h / io.h の手書きFFIバインディング

use std::os::raw::{c_int, c_long, c_uchar, c_uint, c_ushort, c_void};

pub type Tmg1ReadFn  = Option<unsafe extern "C" fn(*mut c_void, *mut c_uchar, usize) -> c_int>;
pub type Tmg1WriteFn = Option<unsafe extern "C" fn(*mut c_void, *const c_uchar, usize) -> c_int>;
pub type Tmg1TellFn  = Option<unsafe extern "C" fn(*mut c_void) -> c_long>;
pub type Tmg1SeekFn  = Option<unsafe extern "C" fn(*mut c_void, c_long, c_int) -> c_int>;

#[repr(C)]
pub struct Tmg1Stream {
    pub ctx:   *mut c_void,
    pub read:  Tmg1ReadFn,
    pub write: Tmg1WriteFn,
    pub tell:  Tmg1TellFn,
    pub seek:  Tmg1SeekFn,
}

#[repr(C)]
pub struct Tmg1EncodeConfig {
    pub width:          c_ushort,
    pub height:         c_ushort,
    pub timebase_num:   c_ushort,
    pub timebase_den:   c_ushort,
    pub key_interval:   c_ushort,
    pub msb_first:      c_uchar,
    pub use_range_coder: c_uchar,
    pub delta_enabled:  c_uchar,
    pub prediction_enabled: c_uchar,
    pub rice_mode:      c_uchar,
    pub rice_k:         c_uchar,
    pub scene_change_enabled: c_uchar,
    pub vfr_enabled:    c_uchar,
    pub index_enabled:  c_uchar,
}

pub enum Tmg1DecoderOpaque {}
pub enum Tmg1EncoderOpaque {}

extern "C" {
    pub fn tmg1_decoder_create(stream: *mut Tmg1Stream) -> *mut Tmg1DecoderOpaque;
    pub fn tmg1_decoder_destroy(dec: *mut Tmg1DecoderOpaque);
    pub fn tmg1_decoder_decode_frame(dec: *mut Tmg1DecoderOpaque, out: *mut c_uchar, out_size: usize) -> c_int;
    pub fn tmg1_decoder_width(dec: *const Tmg1DecoderOpaque) -> c_ushort;
    pub fn tmg1_decoder_height(dec: *const Tmg1DecoderOpaque) -> c_ushort;
    pub fn tmg1_decoder_timebase_num(dec: *const Tmg1DecoderOpaque) -> c_ushort;
    pub fn tmg1_decoder_timebase_den(dec: *const Tmg1DecoderOpaque) -> c_ushort;
    pub fn tmg1_decoder_last_pts_delta(dec: *const Tmg1DecoderOpaque) -> c_uint;

    pub fn tmg1_encoder_create(stream: *mut Tmg1Stream, config: *const Tmg1EncodeConfig) -> *mut Tmg1EncoderOpaque;
    pub fn tmg1_encoder_destroy(enc: *mut Tmg1EncoderOpaque);
    pub fn tmg1_encoder_encode_frame(enc: *mut Tmg1EncoderOpaque, frame: *const c_uchar, frame_size: usize) -> c_int;
    pub fn tmg1_encoder_finish(enc: *mut Tmg1EncoderOpaque) -> c_int;
}
