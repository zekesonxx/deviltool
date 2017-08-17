# deviltool

deviltool is an extensive toolset for working with files from the game [Devil Daggers](https://store.steampowered.com/app/422970/ "Steam link").

## Progress
* [x] Basic (get to feature parity with the C version)
    * [x] Decode main file headers
    * [x] Decode file entries
    * [x] Extract files
* [ ] Better Extraction
    * [x] Detect and categorize all known file types  
      currently `0x01`, `0x02`, `0x10`, `0x11`, `0x20`, and `0x80`
    * [x] Add extensions automatically when extracting
    * [x] Folder marker things
    * [x] Split GLSL files into their respective vert and frag files
    * [ ] Decode dd_tex2 (magic number 0x11 0x40) files into bmps or something
    * [ ] Figure out how the folders really work
    * [ ] Figure out what dd_tex1 is
* [ ] Packing
    * [ ] Basic packing
    * [ ] Repack the two glsl shaders into one file
    * [ ] Repack bmp into dd_tex2 or something
    * [ ] Folders?

## Explanationy
The original work on this was done in [McKay42/devil-daggers-extractor](https://github.com/McKay42/devil-daggers-extractor). However, it had a number of problems:

* `./dd-ext audio audiofolder` (no trailing slash) would result in a bunch of files named "audiofolderandrasimpact.wav", "audiofolderandrasrise.wav", etc; instead of extracting them to a folder called "audiofolder".
* When it extracted, it would step on itself and override files, since names aren't unique (a name and type seem to be unique, however)
    * For example, `dd` has three different files named "boid", however they're all different types.
* He added the length of the header to the header length, thereby making it an offset, and then proceeded to comment about the "redundant" offset.

## FAQ
Q. Why?  
A. You know, that's a really good question. Especially since I've spent <10 min playing the game.
