# Parquet2

This is a re-write of the official [`parquet` crate](https://crates.io/crates/parquet) with performance, parallelism and safety in mind.

The four main differentiators in comparison with `parquet` are:
* does not use `unsafe`
* delegates parallelism downstream
* decouples reading (IO intensive) from computing (CPU intensive)
* deletages decompressing and decoding batches downstream
* it is faster (10-20x when reading to arrow format)

## Organization

* `read`: read metadata and pages
* `write`: write metadata and pages
* `metadata`: parquet files metadata (e.g. `FileMetaData`)
* `schema`: types metadata declaration (e.g. `ConvertedType`)
* `types`: physical type declaration (i.e. how things are represented in memory). So far unused.
* `compression`: compression (e.g. Gzip)
* `error`: errors declaration
* `serialization`: convert from bytes to rust native typed buffers (`Vec<Option<T>>`).

Note that `serialization` is not very robust. It serves as a playground 
to understand the specification and how to serialize and deserialize pages.

## How to use

```rust
use std::fs::File;

use parquet2::read::{Page, read_metadata, get_page_iterator};

let mut file = File::open("testing/parquet-testing/data/alltypes_plain.parquet").unwrap();

/// here we read the metadata.
let metadata = read_metadata(&mut file)?;

/// Here we get an iterator of pages (each page has its own data)
/// This can be heavily parallelized; not even the same `file` is needed here...
/// feel free to wrap `metadata` under an `Arc`
let row_group = 0;
let column = 0;
let mut iter = get_page_iterator(&metadata, row_group, column, &mut file)?;

/// A page. It is just (compressed) bytes at this point.
let page = iter.next().unwrap().unwrap();
println!("{:#?}", page);

/// from here, we can do different things. One of them is to convert its buffers to native Rust.
/// This consumes the page.
use parquet2::serialization::native::page_to_array;
let array = page_to_array(page, &descriptor).unwrap();
```

### How to implement page readers

In general, the in-memory format used to consume parquet pages strongly influences how the pages
should be deserialized. As such, this crate does not commit to a particular in-memory format.Consumers are responsible for converting pages to their target in-memory format.

There is an implementation that uses the arrow format [here](https://github.com/jorgecarleitao/arrow2).

### Higher Parallelism

The function above creates an iterator over a row group, `iter`. In arrow, this
corresponds to a `RecordBatch`, divided in Parquet pages. Typically, 
converting a page into in-memory is expensive and thus consider how to 
distribute work across threads. E.g.

```rust 
let handles = vec![];
for column in columns {
    let compressed_pages = get_page_iterator(&metadata, row_group, column, &mut file, file)?.collect()?;
    // each compressed_page has a buffer; cloning is expensive(!). We move it so that the memory
    // is released at the end of the processing.
    handles.push(thread::spawn move {
        page_iter_to_array(compressed_pages.into_iter())
    })
}
let columns_from_all_groups = handles.join_all();
```

this will read the file as quickly as possible in the main thread and send CPU-intensive work to other threads, thereby maximizing IO reads (at the cost of storing multiple compressed pages in memory; buffering is also an option here).

## General data flow

`parquet -> decompress -> decode -> deserialize`

* `decompress`: e.g. `gzip`
* `decode`: e.g. `RLE`
* `deserialize`: e.g. `&[u8] -> &[i32]`

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
