# Arbitrator

Apply rules to text when lines match regexes.

## Installation

```shell
cargo install arbitrator
```

## Usage

**example.txt**

```txt
This is one of my paintings.
Wow thats really cool.
Thank you!
```

**example.json**

```json
{
    "paintings": "cool {} nice",
    "cool": "{} {}"
}
```

Example command:

```shell
arbitrator --input example.txt --rules example.json --output output.txt
```

**output.txt**

```txt
cool This is one of my paintings. nice
Wow thats really cool. Thank you!
```
