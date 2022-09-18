use std::collections::HashSet;
use std::io::{Result, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::fs::File;

use byteorder::{ReadBytesExt, ByteOrder, BigEndian, LittleEndian};
use std::collections::{HashMap};
use enum_primitive::FromPrimitive;

// use tiff::{TIFF, IFD, IFDEntry, decode_tag, decode_tag_type};


/// The basic TIFF struct. This includes the header (specifying byte order and IFD offsets) as
/// well as all the image file directories (IFDs) plus image data.
///
/// The image data has a size of width * length * bytes_per_sample.
#[derive(Debug)]
pub struct TIFF {
    pub ifds: Vec<IFD>,
    // This is width * length * bytes_per_sample.
    pub image_data: Vec<Vec<Vec<usize>>>,
}

/// The header of a TIFF file. This comes first in any TIFF file and contains the byte order
/// as well as the offset to the IFD table.
#[derive(Debug)]
pub struct TIFFHeader {
    pub byte_order: TIFFByteOrder,
    pub ifd_offset: LONG,
}

/// An image file directory (IFD) within this TIFF. It contains the number of individual IFD entries
/// as well as a Vec with all the entries.
#[derive(Debug)]
pub struct IFD {
    pub count:   u16,
    pub entries: Vec<IFDEntry>,
}

/// A single entry within an image file directory (IDF). It consists of a tag, a type, and several
/// tag values.
#[derive(Debug)]
pub struct IFDEntry {
    pub tag:          TIFFTag,
    pub tpe:          TagType,
    pub count:        LONG,
    pub value_offset: LONG,
    pub value:        Vec<TagValue>,
}

/// Implementations for the IFD struct.
impl IFD {
    pub fn get_image_length() -> usize {
        3
    }

    pub fn get_image_width() -> usize {
        3
    }

    pub fn get_bytes_per_sample() -> usize {
        3
    }
}

/// Decodes an u16 value into a TIFFTag.
pub fn decode_tag(value: u16) -> Option<TIFFTag> {
    TIFFTag::from_u16(value)
}

/// Decodes an u16 value into a TagType.
pub fn decode_tag_type(tpe: u16) -> Option<TagType> {
    TagType::from_u16(tpe)
}

/// Validation functions to make sure all the required tags are existing for a certain GeoTiff
/// image type (e.g., grayscale or RGB image).
pub fn validate_required_tags_for(typ: &ImageType) -> Option<HashSet<TIFFTag>> {
    let required_grayscale_tags: HashSet<TIFFTag> = [
        TIFFTag::ImageWidthTag,
        TIFFTag::ImageLengthTag,
        TIFFTag::BitsPerSampleTag,
        TIFFTag::CompressionTag,
        TIFFTag::PhotometricInterpretationTag,
        TIFFTag::StripOffsetsTag,
        TIFFTag::RowsPerStripTag,
        TIFFTag::StripByteCountsTag,
        TIFFTag::XResolutionTag,
        TIFFTag::YResolutionTag,
        TIFFTag::ResolutionUnitTag].iter().cloned().collect();

    let required_rgb_image_tags: HashSet<TIFFTag> = [
        TIFFTag::ImageWidthTag,
        TIFFTag::ImageLengthTag,
        TIFFTag::BitsPerSampleTag,
        TIFFTag::CompressionTag,
        TIFFTag::PhotometricInterpretationTag,
        TIFFTag::StripOffsetsTag,
        TIFFTag::SamplesPerPixelTag,
        TIFFTag::RowsPerStripTag,
        TIFFTag::StripByteCountsTag,
        TIFFTag::XResolutionTag,
        TIFFTag::YResolutionTag,
        TIFFTag::ResolutionUnitTag,
    ].iter().cloned().collect();

    match *typ {
        ImageType::Bilevel => None,
        ImageType::Grayscale => None,
        ImageType::PaletteColour => None,
        ImageType::RGB => Some(required_rgb_image_tags.difference(&required_grayscale_tags).cloned().collect()),
        ImageType::YCbCr => None,
    }
}

pub struct TIFFStream {
    pub ifds: Vec<IFD>,
    pub offsets: Vec<u32>,
    pub byte_counts: Vec<u32>,
    pub image_depth: u16,
    pub path: Option<PathBuf>,
}

impl TIFFStream {
    /// Opens a `.tiff` file at the location indicated by `filename`.
    pub fn open(filename: &str) -> Result<TIFFStream> {
        let tiff_reader = TIFFReader;
        let mut stream = tiff_reader.load(filename).unwrap();
        Ok(stream)
    }

    /// Gets the value at a given coordinate (in pixels).
    pub fn get_value_at(&self, lon: usize, lat: usize) -> usize {
        // self.image_data[lon][lat][0]
        let mut reader = File::open(self.path.clone().unwrap()).unwrap();
        /*for j in 0..100 {
            reader.seek(SeekFrom::Start((self.offsets[lon as usize]) as u64 /*+ ((byte_counts[3 as usize] / image_depth as u32) * 1) as u64*/)).unwrap();
            /*for i in 0..lat {
                self.read_n(&mut reader, (self.image_depth) as u64);
            }*/
            self.read_n(&mut reader, lat as u64);
            let v = self.read_n(&mut reader, (self.image_depth) as u64);
            self.vec_to_value::<LittleEndian>(v);
        }*/
        reader.seek(SeekFrom::Start((self.offsets[lon as usize]) as u64 /*+ ((byte_counts[3 as usize] / image_depth as u32) * 1) as u64*/)).unwrap();
            /*for i in 0..lat {
                self.read_n(&mut reader, (self.image_depth) as u64);
            }*/
        self.read_n(&mut reader, self.image_depth as u64 * lat as u64); // y offset
        let v = self.read_n(&mut reader, (self.image_depth) as u64);
        self.vec_to_value::<LittleEndian>(v)
    }

    fn read_n(&self, reader: &mut dyn SeekableReader, bytes_to_read: u64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(bytes_to_read as usize);
        let mut chunk = reader.take(bytes_to_read);
        
        let status = chunk.read_to_end(&mut buf);
        // println!("{:?}", buf);
        match status {
            Ok(n) => assert_eq!(bytes_to_read as usize, n),
            _ => panic!("Didn't read enough"),
        }
        buf
    }

    fn vec_to_value<Endian: ByteOrder>(&self, vec: Vec<u8>) -> usize {
        let len = vec.len();
        match len {
            0 => 0 as usize,
            1 => vec[0] as usize,
            2 => Endian::read_u16(&vec[..]) as usize,
            4 => Endian::read_u32(&vec[..]) as usize,
            8 => Endian::read_u64(&vec[..]) as usize,
            _ => panic!("Vector has wrong number of elements!"),
        }
    }
}

/// A helper trait to indicate that something needs to be seekable and readable.
pub trait SeekableReader: Seek + Read {}

impl<T: Seek + Read> SeekableReader for T {}

/// The TIFF reader class that encapsulates all functionality related to reading `.tiff` files.
/// In particular, this includes reading the TIFF header, the image file directories (IDF), and
/// the plain data.
pub struct TIFFReader;

impl TIFFReader {
    /// Loads a `.tiff` file, as specified by `filename`.
    pub fn load(&self, filename: &str) -> Result<TIFFStream> {
        let filepath = Path::new(filename);
        let mut reader = File::open(&filepath)?;

        let mut result = *self.read(&mut reader).unwrap();
        result.path = Some(filepath.to_path_buf().clone());
        Ok(result)
    }

    /// Reads the `.tiff` file, starting with the byte order.
    pub fn read(&self, reader: &mut dyn SeekableReader) -> Result<Box<TIFFStream>> {
        match self.read_byte_order(reader)? {
            TIFFByteOrder::LittleEndian => self.read_tiff::<LittleEndian>(reader),
            TIFFByteOrder::BigEndian => self.read_tiff::<BigEndian>(reader),
        }
    }

    /// Helper function to read the byte order, one of `LittleEndian` or `BigEndian`.
    pub fn read_byte_order(&self, reader: &mut dyn SeekableReader) -> Result<TIFFByteOrder> {
        // Bytes 0-1: "II" or "MM"
        // Read and validate ByteOrder
        match TIFFByteOrder::from_u16(reader.read_u16::<LittleEndian>()?) {
            Some(TIFFByteOrder::LittleEndian) => Ok(TIFFByteOrder::LittleEndian),
            Some(TIFFByteOrder::BigEndian) => Ok(TIFFByteOrder::BigEndian),
            None => Err(Error::new(ErrorKind::Other, format!("Invalid byte order in header."))),
        }
    }

    /// Reads the `.tiff` file, given a `ByteOrder`.
    ///
    /// This starts by reading the magic number, the IFD offset, the IFDs themselves, and finally,
    /// the image data.
    fn read_tiff<T: ByteOrder>(&self, reader: &mut dyn SeekableReader) -> Result<Box<TIFFStream>> {
        self.read_magic::<T>(reader)?;
        let ifd_offset = self.read_ifd_offset::<T>(reader)?;
        let ifd = self.read_IFD::<T>(reader, ifd_offset)?;
        println!("read_IFD done");
        let (offsets, byte_counts, image_depth) = self.read_image_data::<T>(reader, &ifd)?;
        Ok(Box::new(TIFFStream {
            ifds: vec![ifd],
            offsets,
            byte_counts,
            image_depth,
            path: None,
        }))
    }

    /// Reads the magic number, i.e., 42.
    fn read_magic<T: ByteOrder>(&self, reader: &mut dyn SeekableReader) -> Result<()> {
        // Bytes 2-3: 0042
        // Read and validate HeaderMagic
        match reader.read_u16::<T>()? {
            42 => Ok(()),
            _ => Err(Error::new(ErrorKind::Other, "Invalid magic number in header")),
        }
    }

    /// Reads the IFD offset. The first IFD is then read from this position.
    pub fn read_ifd_offset<T: ByteOrder>(&self, reader: &mut dyn SeekableReader) -> Result<u32> {
        // Bytes 4-7: offset
        // Offset from start of file to first IFD
        let ifd_offset_field = reader.read_u32::<T>()?;
        // println!("IFD offset: {:?}", ifd_offset_field);
        Ok(ifd_offset_field)
    }

    /// Reads an IFD.
    ///
    /// This starts by reading the number of entries, and then the tags within each entry.
    #[allow(non_snake_case)]
    fn read_IFD<T: ByteOrder>(&self, reader: &mut dyn SeekableReader, ifd_offset: u32) -> Result<IFD> {
        reader.seek(SeekFrom::Start(ifd_offset as u64))?;
        // 2 byte count of IFD entries
        let entry_count = reader.read_u16::<T>()?;

        // println!("IFD entry count: {}", entry_count);

        let mut ifd = IFD { count: entry_count, entries: Vec::with_capacity(entry_count as usize) };

        for entry_number in 0..entry_count as usize {
            let entry = self.read_tag::<T>(ifd_offset as u64 + 2, entry_number, reader);
            match entry {
                Ok(e) => ifd.entries.push(e),
                Err(err) => println!("Invalid tag at index {}: {}", entry_number, err),
            }
        }

        Ok(ifd)
    }

    /// Reads `n` bytes from a reader into a Vec<u8>.
    fn read_n(&self, reader: &mut dyn SeekableReader, bytes_to_read: u64) -> Vec<u8> {
        let mut buf = Vec::with_capacity(bytes_to_read as usize);
        let mut chunk = reader.take(bytes_to_read);
        
        let status = chunk.read_to_end(&mut buf);
        // println!("{:?}", buf);
        match status {
            Ok(n) => assert_eq!(bytes_to_read as usize, n),
            _ => panic!("Didn't read enough"),
        }
        buf
    }

    /// Converts a Vec<u8> into a TagValue, depending on the type of the tag. In the TIFF file
    /// format, each tag type indicates which value it stores (e.g., a byte, ascii, or long value).
    /// This means that the tag values have to be read taking the tag type into consideration.
    fn vec_to_tag_value<Endian: ByteOrder>(&self, vec: Vec<u8>, tpe: &TagType) -> TagValue {
        let len = vec.len();
        match tpe {
            &TagType::ByteTag => TagValue::ByteValue(vec[0]),
            &TagType::ASCIITag => TagValue::AsciiValue(String::from_utf8_lossy(&vec).to_string()),
            &TagType::ShortTag => TagValue::ShortValue(Endian::read_u16(&vec[..])),
            &TagType::LongTag => TagValue::LongValue(Endian::read_u32(&vec[..])),
            &TagType::RationalTag => TagValue::RationalValue((Endian::read_u32(&vec[..(len / 2)]),
                                                              Endian::read_u32(&vec[(len / 2)..]))),
            &TagType::SignedByteTag => TagValue::SignedByteValue(vec[0] as i8),
            &TagType::SignedShortTag => TagValue::SignedShortValue(Endian::read_i16(&vec[..])),
            &TagType::SignedLongTag => TagValue::SignedLongValue(Endian::read_i32(&vec[..])),
            &TagType::SignedRationalTag => TagValue::SignedRationalValue((Endian::read_i32(&vec[..(len / 2)]),
                                                                          Endian::read_i32(&vec[(len / 2)..]))),
            &TagType::FloatTag => TagValue::FloatValue(Endian::read_f32(&vec[..])),
            &TagType::DoubleTag => TagValue::DoubleValue(Endian::read_f64(&vec[..])),
            &TagType::UndefinedTag => TagValue::ByteValue(0),
            _ => panic!("Tag not found!"),
        }
    }

    /// Converts a number of u8 values to a usize value. This doesn't check if usize is at least
    /// u64, so be careful with large values.
    fn vec_to_value<Endian: ByteOrder>(&self, vec: Vec<u8>) -> usize {
        let len = vec.len();
        match len {
            0 => 0 as usize,
            1 => vec[0] as usize,
            2 => Endian::read_u16(&vec[..]) as usize,
            4 => Endian::read_u32(&vec[..]) as usize,
            8 => Endian::read_u64(&vec[..]) as usize,
            _ => panic!("Vector has wrong number of elements!"),
        }
    }

    /// Reads a single tag (given an IFD offset) into an IFDEntry.
    ///
    /// This consists of reading the tag ID, field type, number of values, offset to values. After
    /// decoding the tag and type, the values are retrieved.
    fn read_tag<Endian: ByteOrder>(&self, ifd_offset: u64, entry_number: usize,
                                   reader: &mut dyn SeekableReader) -> Result<IFDEntry> {
        // println!("Reading tag at {}/{}", ifd_offset, entry_number);
        // Seek beginning (as each tag is 12 bytes long).
        reader.seek(SeekFrom::Start(ifd_offset + 12 * entry_number as u64))?;

        // Bytes 0..1: u16 tag ID
        let tag_value = reader.read_u16::<Endian>()?;

        // Bytes 2..3: u16 field Type
        let tpe_value = reader.read_u16::<Endian>()?;

        // Bytes 4..7: u32 number of Values of type
        let count_value = reader.read_u32::<Endian>()?;

        // Bytes 8..11: u32 offset in file to Value
        let value_offset_value = reader.read_u32::<Endian>()?;

        // Decode the tag.
        let tag_msg = format!("Invalid tag {:04X}", tag_value);
        let tag = decode_tag(tag_value).ok_or(Error::new(ErrorKind::InvalidData, tag_msg))?;

        // Decode the type.
        let tpe_msg = format!("Invalid tag type {:04X}", tpe_value);
        let tpe = decode_tag_type(tpe_value).expect(&tpe_msg);
        let value_size = tag_size(&tpe);

        // Let's get the value(s) of this tag.
        let tot_size = count_value * value_size;
        // println!("{:04X} {:04X} {:08X} {:08X} {:?} {:?} {:?} {:?}", tag_value, tpe_value,
        //        count_value, value_offset_value, tag, tpe, value_size, tot_size);

        let mut values = Vec::with_capacity(count_value as usize);
        if tot_size <= 4 {
            // Can directly read the value at the value field. For simplicity, we simply reset
            // the reader to the correct position.
            reader.seek(SeekFrom::Start(ifd_offset + 12 * entry_number as u64 + 8))?;
            for _ in 0..count_value as usize {
                let value = self.read_n(reader, value_size as u64);
                values.push(self.vec_to_tag_value::<Endian>(value, &tpe));
            }
        } else {
            // Have to read from the address pointed at by the value field.
            reader.seek(SeekFrom::Start(value_offset_value as u64))?;
            for _ in 0..count_value as usize {
                let value = self.read_n(reader, value_size as u64);
                values.push(self.vec_to_tag_value::<Endian>(value, &tpe));
            }
        }

        // Create IFD entry.
        let ifd_entry = IFDEntry {
            tag,
            tpe,
            count: count_value,
            value_offset: value_offset_value,
            value: values,
        };

        // println!("IFD[{:?}] tag: {:?} type: {:?} count: {} offset: {:08x} value: {:?}",
        //         entry_number, ifd_entry.tag, ifd_entry.tpe, ifd_entry.count,
        //         ifd_entry.value_offset, ifd_entry.value);

        Ok(ifd_entry)
    }

    /// Reads the image data into a 3D-Vec<u8>.
    ///
    /// As for now, the following assumptions are made:
    /// * No compression is used, i.e., CompressionTag == 1.
    fn read_image_data<Endian: ByteOrder>(&self, reader: &mut dyn SeekableReader,
                                          ifd: &IFD) -> Result<(Vec<u32>, Vec<u32>, u16)> {
        // Image size and depth.
        let image_length = ifd.entries.iter().find(|&e| e.tag == TIFFTag::ImageLengthTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Image length not found."))?;
        let image_width = ifd.entries.iter().find(|&e| e.tag == TIFFTag::ImageWidthTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Image width not found."))?;
        let image_depth = ifd.entries.iter().find(|&e| e.tag == TIFFTag::BitsPerSampleTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Image depth not found."))?;

        // Storage location within the TIFF. First, lets get the number of rows per strip.
        /* let rows_per_strip = ifd.entries.iter().find(|&e| e.tag == TIFFTag::RowsPerStripTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Rows per strip not found."))?;*/
        // For each strip, its offset within the TIFF file.
        let strip_offsets = ifd.entries.iter().find(|&e| e.tag == TIFFTag::StripOffsetsTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Strip offsets not found."))?;
        let strip_byte_counts = ifd.entries.iter().find(|&e| e.tag == TIFFTag::StripByteCountsTag)
            .ok_or(Error::new(ErrorKind::InvalidData, "Strip byte counts not found."))?;

        // Create the output Vec.
        let image_length = match image_length.value[0] {
            TagValue::ShortValue(v) => v,
            _ => 0 as u16,
        };
        let image_width = match image_width.value[0] {
            TagValue::ShortValue(v) => v,
            _ => 0 as u16,
        };
        let image_depth = match image_depth.value[0] {
            TagValue::ShortValue(v) => v / 8,
            _ => 0 as u16,
        };
        // TODO The img Vec should optimally not be of usize, but of size "image_depth".
        println!("here 1");
        // let mut img: Vec<Vec<Vec<usize>>> = Vec::with_capacity(image_length as usize);
        println!("here 2");
        /*for i in 0..image_length {
            &img.push(Vec::with_capacity(image_width as usize));
            for j in 0..image_width {
                &img[i as usize].push(vec![0; 1]); // TODO To be changed to take into account SamplesPerPixel!
            }
        }*/
        println!("here 3");

        // Read strip after strip, and copy it into the output Vec.
        /*
        let rows_per_strip = match rows_per_strip.value[0] {
            TagValue::ShortValue(v) => v,
            _ => 0 as u16,
        };*/
        let mut offsets: Vec<u32> = Vec::with_capacity(strip_offsets.value.len());
        for v in &strip_offsets.value {
            match v {
                TagValue::LongValue(v) => offsets.push(*v),
                _ => (),
            };
        }
        println!("here 4");
        let mut byte_counts: Vec<u32> = Vec::with_capacity(strip_byte_counts.value.len());
        for v in &strip_byte_counts.value {
            match v {
                TagValue::LongValue(v) => byte_counts.push(*v),
                _ => (),
            };
        }
        println!("here 5");

        Ok((offsets, byte_counts, image_depth))
        // A bit much boilerplate, but should be okay and fast.
        /*
        let mut curr_x = 0;
        let mut curr_y = 0;
        let mut curr_z = 0;
        

        reader.seek(SeekFrom::Start((offsets[3 as usize]) as u64 + (byte_counts[3 as usize] / image_depth as u32 * 0) as u64))?;
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 3, y: 0 => {}", self.vec_to_value::<Endian>(v));
        // println!("byte pos: {}", offsets[3 as usize] as u64 + (byte_counts[3 as usize] / image_depth as u32 * 0) as u64);

        reader.seek(SeekFrom::Start((offsets[3 as usize]) as u64 /*+ ((byte_counts[3 as usize] / image_depth as u32) * 1) as u64*/))?;
        for i in 0..0 {
            self.read_n(reader, (image_depth) as u64);
        }
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 3, y: 1 => {}", self.vec_to_value::<Endian>(v));

        reader.seek(SeekFrom::Start((offsets[3 as usize]) as u64 /*+ ((byte_counts[3 as usize] / image_depth as u32) * 2) as u64*/))?;
        for i in 0..1 {
            self.read_n(reader, (image_depth) as u64);
        }
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 3, y: 2 => {}", self.vec_to_value::<Endian>(v));

        reader.seek(SeekFrom::Start((offsets[3 as usize]) as u64 + ((byte_counts[3 as usize] / image_depth as u32) * 3) as u64))?;
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 3, y: 3 => {}", self.vec_to_value::<Endian>(v));

        reader.seek(SeekFrom::Start((offsets[3 as usize]) as u64 + ((byte_counts[3 as usize] / image_depth as u32) * 4) as u64))?;
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 3, y: 4 => {}", self.vec_to_value::<Endian>(v));

        reader.seek(SeekFrom::Start((offsets[23 as usize]) as u64/* + ((byte_counts[23 as usize] / image_depth as u32 * 3470)) as u64*/))?;
        for i in 0..3470 {
            self.read_n(reader, (image_depth) as u64);
        }
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 23, y: 3470 => {}", self.vec_to_value::<Endian>(v));

        reader.seek(SeekFrom::Start((offsets[23 as usize]) as u64/* + (byte_counts[23 as usize] / image_depth as u32 * 3464) as u64*/))?;
        for i in 0..3464 {
            self.read_n(reader, (image_depth) as u64);
        }
        let v = self.read_n(reader, (image_depth) as u64);
        println!("x: 23, y: 3464 => {}", self.vec_to_value::<Endian>(v));
        
        for (h, (offset, byte_count)) in offsets.iter().zip(byte_counts.iter()).enumerate() {
            // println!("offset: {}, byte_count: {}", offset, byte_count);
            // println!("x: {}, y: {}", curr_x, curr_y);
            reader.seek(SeekFrom::Start(*offset as u64))?;
            for i in 0..(*byte_count / image_depth as u32) {
                // println!("x {:?} len {:?}", curr_x, img.len());
                // println!("y {:?} wid {:?}", curr_y, img[0].len());
                // println!("z {:?} dep {:?}", curr_z, img[0][0].len());

                // let v = self.read_n(reader, image_depth as u64);
                let v = self.read_n(reader, (image_depth) as u64);
                // println!("{:?}", v);
                if (curr_x == 3 && curr_y == 0) {
                    println!("h: {}, i: {}", h, i);
                    println!("x: 3, y: 0 => {}", self.vec_to_value::<Endian>(v));
                } else if (curr_x == 3 && curr_y == 2) {
                    println!("h: {}, i: {}", h, i);
                    println!("x: 3, y: 2 => {}", self.vec_to_value::<Endian>(v));
                } else if (curr_x == 23 && curr_y == 3470) {
                    println!("h: {}, i: {}", h, i);
                    println!("x: 23, y: 3470 => {}", self.vec_to_value::<Endian>(v));
                } else if (curr_x == 23 && curr_y == 3464) {
                    println!("h: {}, i: {}", h, i);
                    println!("x: 23, y: 3464 => {}", self.vec_to_value::<Endian>(v));
                }
                // img[curr_x][curr_y][0] = self.vec_to_value::<Endian>(v);
                
                curr_z += 1;


                if curr_z >= img[curr_x][curr_y].len() {
                    curr_z = 0;
                    curr_y += 1;
                }
                if curr_y >= img[curr_x].len() as usize {
                    curr_y = 0;
                    curr_x += 1;
                }
            }
        }
        */

        // Return the output Vec.
        // Ok(img)
    }
}


// Base types of the TIFF format.
pub type BYTE      = u8;
pub type SHORT     = u16;
pub type LONG      = u32;
pub type ASCII     = String;
pub type RATIONAL  = (u32, u32);
pub type SBYTE     = i8;
pub type SSHORT    = i16;
pub type SLONG     = i32;
pub type SRATIONAL = (i32, i32);
pub type FLOAT     = f32;
pub type DOUBLE    = f64;

// Different values individual components can take.
enum_from_primitive! {
    #[repr(u16)]
    #[derive(Debug)]
    pub enum TIFFByteOrder {
        LittleEndian = 0x4949,
        BigEndian    = 0x4d4d,
    }
}

enum_from_primitive! {
    #[repr(u16)]
    #[derive(Debug,PartialEq)]
    pub enum TagType {
        ByteTag           = 1,
        ASCIITag          = 2,
        ShortTag          = 3,
        LongTag           = 4,
        RationalTag       = 5,
        SignedByteTag     = 6,
        UndefinedTag      = 7,
        SignedShortTag    = 8,
        SignedLongTag     = 9,
        SignedRationalTag = 10,
        FloatTag          = 11,
        DoubleTag         = 12,
    }
}

/// Helper function that returns the size of a certain tag.
pub fn tag_size(t: &TagType) -> u32 {
    match *t {
        TagType::ByteTag           => 1,
        TagType::ASCIITag          => 1,
        TagType::ShortTag          => 2,
        TagType::LongTag           => 4,
        TagType::RationalTag       => 8,
        TagType::SignedByteTag     => 1,
        TagType::UndefinedTag      => 1,
        TagType::SignedShortTag    => 2,
        TagType::SignedLongTag     => 2,
        TagType::SignedRationalTag => 8,
        TagType::FloatTag          => 4,
        TagType::DoubleTag         => 8,
        _                          => 0,
    }
}

/// All the possible values of tags.
#[derive(Debug)]
pub enum TagValue {
    ByteValue(BYTE),
    AsciiValue(ASCII),
    ShortValue(SHORT),
    LongValue(LONG),
    RationalValue(RATIONAL),
    SignedByteValue(SBYTE),
    SignedShortValue(SSHORT),
    SignedLongValue(SLONG),
    SignedRationalValue(SRATIONAL),
    FloatValue(FLOAT),
    DoubleValue(DOUBLE),
}

/// The photometric interpretation of the GeoTIFF.
#[repr(u16)]
#[derive(Debug)]
pub enum PhotometricInterpretation {
    WhiteIsZero = 0,
    BlackIsZero = 1,
}

/// The compression chosen for this TIFF.
#[repr(u16)]
#[derive(Debug)]
pub enum Compression {
    None     = 1,
    Huffman  = 2,
    LZW      = 5,
    OJPEG    = 6,
    JPEG     = 7,
    PackBits = 32773,
}

/// The resolution unit of this TIFF.
#[repr(u16)]
#[derive(Debug)]
pub enum ResolutionUnit {
    None       = 1,
    Inch       = 2,
    Centimetre = 3,
}

/// The sample format of this TIFF.
#[repr(u16)]
#[derive(Debug)]
pub enum SampleFormat {
    UnsignedInteger             = 1,
    TwosComplementSignedInteger = 2,
    IEEEFloatingPoint           = 3,
    Undefined                   = 4,
}

/// The image type of this TIFF.
#[derive(Debug)]
pub enum ImageType {
    Bilevel,
    Grayscale,
    PaletteColour,
    RGB,
    YCbCr,
}

/// The image orientation of this TIFF.
#[repr(u16)]
#[derive(Debug)]
pub enum ImageOrientation {
    TopLeft     = 1,	// row 0 top, col 0 lhs
    TopRight    = 2,	// row 0 top, col 0 rhs
    BottomRight = 3,	// row 0 bottom, col 0 rhs
    BottomLeft  = 4,	// row 0 bottom, col 0 lhs
    LeftTop     = 5,	// row 0 lhs, col 0 top
    RightTop    = 6, 	// row 0 rhs, col 0 top
    RightBottom = 7,	// row 0 rhs, col 0 bottom
    LeftBottom  = 8,	// row 0 lhs, col 0 bottom
}


// Baseline Tags
enum_from_primitive! {
    #[repr(u16)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum TIFFTag {

        // Baseline Tags
        ArtistTag                    = 0x013b,
        BitsPerSampleTag             = 0x0102,
        CellLengthTag                = 0x0109,
        CellWidthTag                 = 0x0108,
        ColorMapTag                  = 0x0140,
        CompressionTag               = 0x0103,
        CopyrightTag                 = 0x8298,
        DateTimeTag                  = 0x0132,
        ExtraSamplesTag              = 0x0152,
        FillOrderTag                 = 0x010a,
        FreeByteCountsTag            = 0x0121,
        FreeOffsetsTag               = 0x0120,
        GrayResponseCurveTag         = 0x0123,
        GrayResponseUnitTag          = 0x0122,
        HostComputerTag              = 0x013c,
        ImageDescriptionTag          = 0x010e,
        ImageLengthTag               = 0x0101,
        ImageWidthTag                = 0x0100,
        MakeTag                      = 0x010f,
        MaxSampleValueTag            = 0x0119,
        MinSampleValueTag            = 0x0118,
        ModelTag                     = 0x0110,
        NewSubfileTypeTag            = 0x00fe,
        OrientationTag               = 0x0112,
        PhotometricInterpretationTag = 0x0106,
        PlanarConfigurationTag       = 0x011c,
        PredictorTag                 = 0x013d,
        ResolutionUnitTag            = 0x0128,
        RowsPerStripTag              = 0x0116,
        SampleFormatTag              = 0x0153,
        SamplesPerPixelTag           = 0x0115,
        SoftwareTag                  = 0x0131,
        StripByteCountsTag           = 0x0117,
        StripOffsetsTag              = 0x0111,
        SubfileTypeTag               = 0x00ff,
        ThresholdingTag              = 0x0107,
        XResolutionTag               = 0x011a,
        YResolutionTag               = 0x011b,

        // Section 20: Colorimetry
        WhitePointTag                = 0x013e,
        PrimaryChromaticities        = 0x013f,
        TransferFunction             = 0x012d,
        TransferRange                = 0x0156,
        ReferenceBlackWhite          = 0x0214,

        // Section 21: YCbCr Images
        YCbCrCoefficients            = 0x0211,
        YCbCrSubsampling             = 0x0212,
        YCbCrPositioning             = 0x0213,

        // TIFF/EP Tags
        SubIFDsTag                   = 0x014a,
        JPEGTablesTag                = 0x015b,
        CFARepeatPatternDimTag       = 0x828d,
        BatteryLevelTag              = 0x828f,
        ModelPixelScaleTag           = 0x830e,
        IPTCTag                      = 0x83BB,
        ModelTiepointTag             = 0x8482,
        ModelTransformationTag       = 0x85D8,
        InterColorProfileTag         = 0x8773,
        GeoKeyDirectoryTag           = 0x87AF,
        GeoDoubleParamsTag           = 0x87B0,
        GeoAsciiParamsTag            = 0x87B1,
        InterlaceTag                 = 0x8829,
        TimeZoneOffsetTag            = 0x882a,
        SelfTimerModeTag             = 0x882b,
        NoiseTag                     = 0x920d,
        ImageNumberTag               = 0x9211,
        SecurityClassificationTag    = 0x9212,
        ImageHistoryTag              = 0x9213,
        EPStandardIdTag              = 0x9216,

        // Extension TIFF Tags
        // See http://www.awaresystems.be/imaging/tiff/tifftags/extension.html
        XMPTag                       = 0x02bc,

        // Private Tags
        PhotoshopTag                 = 0x8649,
        EXIFTag                      = 0x8769,

        GDALMETADATA                 = 0xA480,
        GDALNODATA                   = 0xA481,
    }
}

// Default Values
static PHOTOMETRIC_INTERPRETATION_SHORT_DEFAULT: SHORT = 1;
static PHOTOMETRIC_INTERPRETATION_LONG_DEFAULT: LONG = 1;