#![feature(alloc_system)]
extern crate alloc_system;
extern crate ffmpeg_sys;

use ffmpeg_sys as ffsys;
use ffmpeg_sys::{AVFormatContext, AVPacket};
use std::{ptr, mem};
use std::ffi::CString;

pub type StreamIndex = i32;

fn main() {
    unsafe {
        let c_src_uri = if let Ok(uri) = CString::new("./moviesample/trailer.mp4") {
            uri
        } else {
            println!("Something happened while trying to convert the source URI to a CString");
            std::process::exit(1)
        };

        ffsys::av_register_all();

        let mut avfc: *mut AVFormatContext = ptr::null_mut();
        if ffsys::avformat_open_input(&mut avfc as *mut *mut _, c_src_uri.as_ptr(), ptr::null_mut(), ptr::null_mut()) != 0 {
            println!("Could not open AVFormatContext");
            std::process::exit(1)
        }

        if ffsys::avformat_find_stream_info(avfc, ptr::null_mut()) < 0 {
            println!("Could not find stream info");
            std::process::exit(1)
        }

        let stream_idx = {
            let avfc_deref = &*avfc;
            let num_streams = avfc_deref.nb_streams;
            let streams = avfc_deref.streams;
            (0..num_streams as StreamIndex).find( |x| {
                let av_stream_deref = &**streams.offset(*x as isize);
                let av_codec_ctx_deref = &*av_stream_deref.codec;  
                av_codec_ctx_deref.codec_type == ffsys::AVMEDIA_TYPE_VIDEO
            }).unwrap()
        };

        let avcc = {
            let avfc_deref = &*avfc;
            let streams = avfc_deref.streams;
            let stream = &**streams.offset(stream_idx as isize);
            stream.codec
        };

        let avc = {
            let codec_id = (*avcc).codec_id;
            let codec = ffsys::avcodec_find_decoder(codec_id);
            if codec.is_null() {
                println!("Could not find decoder");
                std::process::exit(1)
            };
            codec
        };

        if ffsys::avcodec_open2(avcc, avc, ptr::null_mut()) < 0 {
            println!("Could not open decoder");
            std::process::exit(1)
        }

        let mut av_packet = TLXAVPacket::new();
        ffsys::av_read_frame(avfc, av_packet.as_mut_ptr());  //Commenting this line will cause the drop to run correctly.

        if !avcc.is_null() {
            ffsys::avcodec_close(avcc);
        }

        if !avfc.is_null() {
            ffsys::avformat_close_input(&mut avfc);
        }
    }

    std::process::exit(0)
}

pub struct TLXAVPacket(AVPacket);

impl TLXAVPacket {

    fn new() -> TLXAVPacket {
        let packet = unsafe {
            let mut pkt: AVPacket = mem::zeroed();
            ffsys::av_init_packet(&mut pkt);
            TLXAVPacket(pkt)
        };

        packet
    }

    fn as_mut_ptr(&mut self) -> *mut AVPacket {
        &mut self.0
    }
}

impl Drop for TLXAVPacket {
    
    fn drop(&mut self) -> () {
        println!("DROPPING PACKET!!!!");
        unsafe {
            ffsys::av_packet_unref(self.as_mut_ptr());
        }
    }
}