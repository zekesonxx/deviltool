# audio/dd/core archive format

This format is used for three files from the game: `core/core`, `res/audio`, and `res/dd`.

The format is quite simple, and should be implementable just about anywhere.

EVERY number in the file is little-endian. It's being said here, so I don't have to keep repeating myself.

## Header
The format starts with the magic bytes ":hx:rg:\x01" (ASCII).
The 01 might be a version number, or it might be padding so it aligns the magic number to 8 bytes.

Following this is a u32, which is the length of the subheader/file list.

## Subheader/file list
This is an list of file information. 

The format here is pretty simple:

    filetype(u16), filename([u8]), null, offset(u32), size(u32), timestamp(u32)


Filename is an array of characters, probably only ASCII.

Timestamp is a Unix timestamp, probably of file creation or modification.
In the real game files these all date to mid-late 2015 and early 2016, which coincides with the game's Feb 2016 release.

Size is the size of the file in bytes.

Offset is the location of the file data within the archive, starting from the last byte after the headers,
with all the files just one after the other (more on this later).

Filetype is a u16 that identifies the type of file it is.
Or maybe it's a u8, and the following byte is used for something else. Dunno.

Known filetypes:

* `0x01`: Some sort of texture or model data. I don't know what this is.
* `0x02`: Texture data (I refer to it as tex2). I can decode them into pngs, but I only understand about half the bytes in the file.
* `0x10`: Combined GLSL vertex and fragment shader. Format of this is below.  
Side note: these all seem to have timestamps of 0.
* `0x11`: Folder marker. Probably. These have length 0, and seem to correspond to how the files might've been organized.
* `0x20`: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, stereo 44100 Hz;
I don't know what the game will or won't accept, but all of the included files are of that format.
* `0x80`: Shader config file. This is a plain text file, I haven't messed with changing them at all yet.


These get repeated for however many files there are, followed by two null bytes.
The entire contents of the file list, including the two null bytes at the end, are counted within the length from the main header.

## Putting it all together
For a basic overview of how a file is put together:

```
:hx:rg:\x01
length of subheader/filesection
file1 info
file2 info
two null bytes
contents of file1
contents of file2
```

## A real example

First, our files:
```
$ ls example/
file1.shadercfg file2.wav
$ cat example/file1.shadercfg
contents of file1
$ cat example/file2.wav
contents of file2
```

Now, let's pack em up:
```
$ deviltool pack examplearchive example
## Building file list
example/file1.shadercfg: shader text file, 17B
example/file2.wav: wav audio, 17B
## Built list of 2 files
Total subheader length: 42B
First file offset at: 54
Beginning file output
Wrote main header
Wrote subheader
Wrote file1
Wrote file2
Built archive examplearchive
```

And take a look:
```
$ hexdump -C examplearchive
00000000  3a 68 78 3a 72 67 3a 01  2a 00 00 00 80 00 66 69  |:hx:rg:.*.....fi|
00000010  6c 65 31 00 36 00 00 00  11 00 00 00 30 b1 96 59  |le1.6.......0..Y|
00000020  20 00 66 69 6c 65 32 00  47 00 00 00 11 00 00 00  | .file2.G.......|
00000030  35 b1 96 59 00 00 63 6f  6e 74 65 6e 74 73 20 6f  |5..Y..contents o|
00000040  66 20 66 69 6c 65 31 63  6f 6e 74 65 6e 74 73 20  |f file1contents |
00000050  6f 66 20 66 69 6c 65 32                           |of file2|
00000058
```
So you don't have to manually transcribe those bytes, I've included examplearchive in the repo.


