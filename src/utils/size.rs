use bytesize::ByteSize;

pub fn format_size(size: usize) -> String {
    ByteSize::b(size as u64).to_string()
}
