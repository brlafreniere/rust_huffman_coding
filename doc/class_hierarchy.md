# The Object Universe

This document describes all of the objects that you might encounter in the
application. It is a class reference, essentially.

# Class Diagram

```mermaid
classDiagram
  class App {
    +invoke(Vec~String~ args)$
  }

  namespace huffman {
    class Encoder {
      +String file_contents
      +new(file_path: String)$ Encoder
      +encode() Vec~u8~
    }

    class Decoder {
      +String file_contents
      +new(file_path: String)$ Decoder
      +decode()
    }
  }
```

## App

Drives the app.

1. Receives program arguments and validates them
2. Determines what actions to take based on program args

There are only two main functions of the program, encoding and decoding.

Two classes exist which drive each process: `huffman::Encoder` and
`huffman::Decoder`.