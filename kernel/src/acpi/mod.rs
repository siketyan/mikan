mod mcfg;
mod rsdp;
mod xsdt;

pub(crate) use mcfg::*;
pub(crate) use rsdp::*;
pub(crate) use xsdt::*;

pub(crate) trait Sdt {}
