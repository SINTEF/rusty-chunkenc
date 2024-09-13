use crate::histogram::{FloatHistogramChunk, HistogramChunk};

impl HistogramChunk {
    pub fn write<W: std::io::Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unimplemented!()
    }
}

impl FloatHistogramChunk {
    pub fn write<W: std::io::Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        unimplemented!()
    }
}
