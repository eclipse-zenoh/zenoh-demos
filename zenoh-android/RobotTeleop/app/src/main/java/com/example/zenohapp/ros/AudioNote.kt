package com.example.zenohapp.ros

import com.example.zenohapp.cdr.CDROutputStream

class AudioNote constructor(var frequency : UShort, var maxRuntime : Time) {

    fun serialize(stream: CDROutputStream) {
        stream.writeUShort(frequency)
        maxRuntime.serialize(stream)
    }
}

class AudioNoteVector constructor(var header: Header, var notes : Array<AudioNote>, var append: Boolean) {

    fun serialize(stream: CDROutputStream) {
        header.serialize(stream)
        stream.writeInt(notes.size)
        for (i in 0..(notes.size-1)) {
            notes[i].serialize(stream)
        }
        stream.writeBool(append)
    }
}