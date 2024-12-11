# The Universe

```mermaid
classDiagram
  class ArgParser {
    +String 
  }

  class Encoder["huffman::Encoder"]
  class Encoder {
    +String file_path
    +String raw_data
    +encode()
  }

  class Decoder["huffman::Decoder"]
  class Decoder {
    +String file_path
    +String encoded_data
    +decode()
  }
```
