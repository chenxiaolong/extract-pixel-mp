# extract-pixel-mp

A simple tool to extract the embedded video from Google Pixel `*.MP.jpg` motion photos (also known as "top shot" photos).

(NOTE: The old `MVIMG_*` format is not supported.)

## Usage

To extract the video from a file and output to the same filename with the extension changed from `.jpg` to `.mp4`, run:

```bash
cargo run -- -i <input>
```

To output to a specific path, pass in `-o <output>`.

To output to stdout, pass in `-O`.

## Format

The new-style motion photos store the size of the embedded video in the XMP metadata:

```xml
<Container:Directory>
  <rdf:Seq>
    <rdf:li rdf:parseType="Resource">
      <Container:Item
        Item:Mime="image/jpeg"
        Item:Semantic="Primary"
        Item:Length="0"
        Item:Padding="0"/>
    </rdf:li>
    <rdf:li rdf:parseType="Resource">
      <Container:Item
        Item:Mime="video/mp4"
        Item:Semantic="MotionPhoto"
        Item:Length="2929880"
        Item:Padding="0"/>
    </rdf:li>
  </rdf:Seq>
</Container:Directory>
```

The data spans from offset `-length` till EOF in the input file.

## License

This project is licensed under GPLv3. Please see [`LICENSE`](./LICENSE) for the full license text.
