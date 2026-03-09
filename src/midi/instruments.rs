pub struct MidiProgramGroup {
    pub name: &'static str,
    pub description: &'static str,
    pub instruments: &'static [u8],
}

pub struct MidiInstruments {
    pub name: &'static str,
    pub group: u8,
    pub range: (u8, u8),
}

pub static MIDI_GROUPS: [MidiProgramGroup; 15] = [
    MidiProgramGroup {
        name: "Piano",
        description: "Acoustic and Electric Pianos",
        instruments: &[0, 1, 2, 3, 4, 5, 6, 7],
    }, // 0
    MidiProgramGroup {
        name: "Chromatic Percussion",
        description: "Tuned Percussion Instruments",
        instruments: &[8, 9, 10, 11, 12, 13, 14, 15],
    }, // 1
    MidiProgramGroup {
        name: "Organ",
        description: "Various Organ Sounds",
        instruments: &[16, 17, 18, 19, 20, 21, 22, 23],
    }, // 2
    MidiProgramGroup {
        name: "Guitar",
        description: "Acoustic and Electric Guitars",
        instruments: &[24, 25, 26, 27, 28, 29, 30, 31],
    }, // 3
    MidiProgramGroup {
        name: "Bass",
        description: "Acoustic and Electric Basses",
        instruments: &[32, 33, 34, 35, 36, 37, 38, 39],
    }, // 4
    MidiProgramGroup {
        name: "Strings",
        description: "String Instruments and Ensembles",
        instruments: &[40, 41, 42, 43, 44, 45, 46, 47],
    }, // 5
    MidiProgramGroup {
        name: "Brass",
        description: "Brass Instruments",
        instruments: &[56, 57, 58, 59, 60, 61, 62, 63],
    }, // 6
    MidiProgramGroup {
        name: "Reed",
        description: "Woodwind Instruments",
        instruments: &[64, 65, 66, 67, 68, 69, 70, 71],
    }, // 7
    MidiProgramGroup {
        name: "Pipe",
        description: "Flutes and Other Pipe Instruments",
        instruments: &[72, 73, 74, 75, 76, 77, 78, 79],
    }, // 8
    MidiProgramGroup {
        name: "Synth Lead",
        description: "Monophonic Synthesizer Leads",
        instruments: &[80, 81, 82, 83, 84, 85, 86, 87],
    }, // 9
    MidiProgramGroup {
        name: "Synth Pad",
        description: "Polyphonic Synthesizer Pads",
        instruments: &[88, 89, 90, 91, 92, 93, 94, 95],
    }, // 10
    MidiProgramGroup {
        name: "Synth Effects",
        description: "Special Synthesizer Effects",
        instruments: &[96, 97, 98, 99, 100, 101, 102, 103],
    }, // 11
    MidiProgramGroup {
        name: "Ethnic",
        description: "Traditional and Ethnic Instruments",
        instruments: &[104, 105, 106, 107, 108, 109, 110, 111],
    }, // 12
    MidiProgramGroup {
        name: "Percussive",
        description: "Unpitched Percussion Instruments",
        instruments: &[112, 113, 114, 115, 116, 117, 118, 119],
    }, // 13
    MidiProgramGroup {
        name: "Sound Effects",
        description: "Various Sound Effects",
        instruments: &[120, 121, 122, 123, 124, 125, 126, 127],
    }, // 14
];

pub static MIDI_INSTRUMENTS: [MidiInstruments; 128] = [
    // Piano
    MidiInstruments {
        name: "Acoustic Grand Piano",
        group: 0,
        range: (21, 108),
    }, // 0
    MidiInstruments {
        name: "Bright Acoustic Piano",
        group: 0,
        range: (21, 108),
    }, // 1
    MidiInstruments {
        name: "Electric Grand Piano",
        group: 0,
        range: (21, 108),
    }, // 2
    MidiInstruments {
        name: "Honky-tonk Piano",
        group: 0,
        range: (21, 108),
    }, // 3
    MidiInstruments {
        name: "Electric Piano 1",
        group: 0,
        range: (28, 103),
    }, // 4
    MidiInstruments {
        name: "Electric Piano 2",
        group: 0,
        range: (28, 103),
    }, // 5
    MidiInstruments {
        name: "Harpsichord",
        group: 0,
        range: (29, 89),
    }, // 6
    MidiInstruments {
        name: "Clavinet",
        group: 0,
        range: (29, 89),
    }, // 7
    // Chromatic Percussion
    MidiInstruments {
        name: "Celesta",
        group: 1,
        range: (60, 108),
    }, // 8
    MidiInstruments {
        name: "Glockenspiel",
        group: 1,
        range: (79, 108),
    }, // 9
    MidiInstruments {
        name: "Music Box",
        group: 1,
        range: (60, 96),
    }, // 10
    MidiInstruments {
        name: "Vibraphone",
        group: 1,
        range: (53, 89),
    }, // 11
    MidiInstruments {
        name: "Marimba",
        group: 1,
        range: (45, 96),
    }, // 12
    MidiInstruments {
        name: "Xylophone",
        group: 1,
        range: (65, 108),
    }, // 13
    MidiInstruments {
        name: "Tubular Bells",
        group: 1,
        range: (60, 77),
    }, // 14
    MidiInstruments {
        name: "Dulcimer",
        group: 1,
        range: (60, 96),
    }, // 15
    // Organ
    MidiInstruments {
        name: "Drawbar Organ",
        group: 2,
        range: (36, 96),
    }, // 16
    MidiInstruments {
        name: "Percussive Organ",
        group: 2,
        range: (36, 96),
    }, // 17
    MidiInstruments {
        name: "Rock Organ",
        group: 2,
        range: (36, 96),
    }, // 18
    MidiInstruments {
        name: "Church Organ",
        group: 2,
        range: (21, 108),
    }, // 19
    MidiInstruments {
        name: "Reed Organ",
        group: 2,
        range: (36, 96),
    }, // 20
    MidiInstruments {
        name: "Accordion",
        group: 2,
        range: (53, 89),
    }, // 21
    MidiInstruments {
        name: "Harmonica",
        group: 2,
        range: (60, 96),
    }, // 22
    MidiInstruments {
        name: "Tango Accordion",
        group: 2,
        range: (53, 89),
    }, // 23
    // Guitar
    MidiInstruments {
        name: "Acoustic Guitar (nylon)",
        group: 3,
        range: (40, 88),
    }, // 24
    MidiInstruments {
        name: "Acoustic Guitar (steel)",
        group: 3,
        range: (40, 88),
    }, // 25
    MidiInstruments {
        name: "Electric Guitar (jazz)",
        group: 3,
        range: (40, 91),
    }, // 26
    MidiInstruments {
        name: "Electric Guitar (clean)",
        group: 3,
        range: (40, 91),
    }, // 27
    MidiInstruments {
        name: "Electric Guitar (muted)",
        group: 3,
        range: (40, 91),
    }, // 28
    MidiInstruments {
        name: "Overdriven Guitar",
        group: 3,
        range: (40, 93),
    }, // 29
    MidiInstruments {
        name: "Distortion Guitar",
        group: 3,
        range: (40, 93),
    }, // 30
    MidiInstruments {
        name: "Guitar Harmonics",
        group: 3,
        range: (40, 88),
    }, // 31
    // Bass
    MidiInstruments {
        name: "Acoustic Bass",
        group: 4,
        range: (28, 67),
    }, // 32
    MidiInstruments {
        name: "Electric Bass (finger)",
        group: 4,
        range: (28, 67),
    }, // 33
    MidiInstruments {
        name: "Electric Bass (pick)",
        group: 4,
        range: (28, 67),
    }, // 34
    MidiInstruments {
        name: "Fretless Bass",
        group: 4,
        range: (28, 67),
    }, // 35
    MidiInstruments {
        name: "Slap Bass 1",
        group: 4,
        range: (28, 67),
    }, // 36
    MidiInstruments {
        name: "Slap Bass 2",
        group: 4,
        range: (28, 67),
    }, // 37
    MidiInstruments {
        name: "Synth Bass 1",
        group: 4,
        range: (21, 96),
    }, // 38
    MidiInstruments {
        name: "Synth Bass 2",
        group: 4,
        range: (21, 96),
    }, // 39
    // Strings
    MidiInstruments {
        name: "Violin",
        group: 5,
        range: (55, 105),
    }, // 40
    MidiInstruments {
        name: "Viola",
        group: 5,
        range: (48, 91),
    }, // 41
    MidiInstruments {
        name: "Cello",
        group: 5,
        range: (36, 76),
    }, // 42
    MidiInstruments {
        name: "Contrabass",
        group: 5,
        range: (28, 55),
    }, // 43
    MidiInstruments {
        name: "Tremolo Strings",
        group: 5,
        range: (36, 96),
    }, // 44
    MidiInstruments {
        name: "Pizzicato Strings",
        group: 5,
        range: (36, 96),
    }, // 45
    MidiInstruments {
        name: "Orchestral Harp",
        group: 5,
        range: (24, 103),
    }, // 46
    MidiInstruments {
        name: "Timpani",
        group: 5,
        range: (36, 57),
    }, // 47
    // Ensemble
    MidiInstruments {
        name: "String Ensemble 1",
        group: 5,
        range: (36, 96),
    }, // 48
    MidiInstruments {
        name: "String Ensemble 2",
        group: 5,
        range: (36, 96),
    }, // 49
    MidiInstruments {
        name: "Synth Strings 1",
        group: 5,
        range: (21, 108),
    }, // 50
    MidiInstruments {
        name: "Synth Strings 2",
        group: 5,
        range: (21, 108),
    }, // 51
    MidiInstruments {
        name: "Choir Aahs",
        group: 5,
        range: (48, 84),
    }, // 52
    MidiInstruments {
        name: "Voice Oohs",
        group: 5,
        range: (48, 84),
    }, // 53
    MidiInstruments {
        name: "Synth Choir",
        group: 5,
        range: (36, 96),
    }, // 54
    MidiInstruments {
        name: "Orchestra Hit",
        group: 5,
        range: (36, 84),
    }, // 55
    // Brass
    MidiInstruments {
        name: "Trumpet",
        group: 6,
        range: (54, 86),
    }, // 56
    MidiInstruments {
        name: "Trombone",
        group: 6,
        range: (40, 72),
    }, // 57
    MidiInstruments {
        name: "Tuba",
        group: 6,
        range: (29, 58),
    }, // 58
    MidiInstruments {
        name: "Muted Trumpet",
        group: 6,
        range: (54, 86),
    }, // 59
    MidiInstruments {
        name: "French Horn",
        group: 6,
        range: (41, 77),
    }, // 60
    MidiInstruments {
        name: "Brass Section",
        group: 6,
        range: (36, 96),
    }, // 61
    MidiInstruments {
        name: "Synth Brass 1",
        group: 6,
        range: (21, 108),
    }, // 62
    MidiInstruments {
        name: "Synth Brass 2",
        group: 6,
        range: (21, 108),
    }, // 63
    // Reed
    MidiInstruments {
        name: "Soprano Sax",
        group: 7,
        range: (56, 88),
    }, // 64
    MidiInstruments {
        name: "Alto Sax",
        group: 7,
        range: (49, 80),
    }, // 65
    MidiInstruments {
        name: "Tenor Sax",
        group: 7,
        range: (44, 76),
    }, // 66
    MidiInstruments {
        name: "Baritone Sax",
        group: 7,
        range: (36, 68),
    }, // 67
    MidiInstruments {
        name: "Oboe",
        group: 7,
        range: (58, 91),
    }, // 68
    MidiInstruments {
        name: "English Horn",
        group: 7,
        range: (52, 81),
    }, // 69
    MidiInstruments {
        name: "Bassoon",
        group: 7,
        range: (34, 72),
    }, // 70
    MidiInstruments {
        name: "Clarinet",
        group: 7,
        range: (50, 91),
    }, // 71
    // Pipe
    MidiInstruments {
        name: "Piccolo",
        group: 8,
        range: (74, 108),
    }, // 72
    MidiInstruments {
        name: "Flute",
        group: 8,
        range: (60, 96),
    }, // 73
    MidiInstruments {
        name: "Recorder",
        group: 8,
        range: (60, 96),
    }, // 74
    MidiInstruments {
        name: "Pan Flute",
        group: 8,
        range: (60, 96),
    }, // 75
    MidiInstruments {
        name: "Blown Bottle",
        group: 8,
        range: (60, 84),
    }, // 76
    MidiInstruments {
        name: "Shakuhachi",
        group: 8,
        range: (55, 84),
    }, // 77
    MidiInstruments {
        name: "Whistle",
        group: 8,
        range: (72, 96),
    }, // 78
    MidiInstruments {
        name: "Ocarina",
        group: 8,
        range: (60, 84),
    }, // 79
    // Synth Lead
    MidiInstruments {
        name: "Lead 1 (square)",
        group: 9,
        range: (0, 127),
    }, // 80
    MidiInstruments {
        name: "Lead 2 (sawtooth)",
        group: 9,
        range: (0, 127),
    }, // 81
    MidiInstruments {
        name: "Lead 3 (calliope)",
        group: 9,
        range: (0, 127),
    }, // 82
    MidiInstruments {
        name: "Lead 4 (chiff)",
        group: 9,
        range: (0, 127),
    }, // 83
    MidiInstruments {
        name: "Lead 5 (charang)",
        group: 9,
        range: (0, 127),
    }, // 84
    MidiInstruments {
        name: "Lead 6 (voice)",
        group: 9,
        range: (0, 127),
    }, // 85
    MidiInstruments {
        name: "Lead 7 (fifths)",
        group: 9,
        range: (0, 127),
    }, // 86
    MidiInstruments {
        name: "Lead 8 (bass + lead)",
        group: 9,
        range: (0, 127),
    }, // 87
    // Synth Pad
    MidiInstruments {
        name: "Pad 1 (new age)",
        group: 10,
        range: (0, 127),
    }, // 88
    MidiInstruments {
        name: "Pad 2 (warm)",
        group: 10,
        range: (0, 127),
    }, // 89
    MidiInstruments {
        name: "Pad 3 (polysynth)",
        group: 10,
        range: (0, 127),
    }, // 90
    MidiInstruments {
        name: "Pad 4 (choir)",
        group: 10,
        range: (0, 127),
    }, // 91
    MidiInstruments {
        name: "Pad 5 (bowed)",
        group: 10,
        range: (0, 127),
    }, // 92
    MidiInstruments {
        name: "Pad 6 (metallic)",
        group: 10,
        range: (0, 127),
    }, // 93
    MidiInstruments {
        name: "Pad 7 (halo)",
        group: 10,
        range: (0, 127),
    }, // 94
    MidiInstruments {
        name: "Pad 8 (sweep)",
        group: 10,
        range: (0, 127),
    }, // 95
    // Synth Effects
    MidiInstruments {
        name: "FX 1 (rain)",
        group: 11,
        range: (0, 127),
    }, // 96
    MidiInstruments {
        name: "FX 2 (soundtrack)",
        group: 11,
        range: (0, 127),
    }, // 97
    MidiInstruments {
        name: "FX 3 (crystal)",
        group: 11,
        range: (0, 127),
    }, // 98
    MidiInstruments {
        name: "FX 4 (atmosphere)",
        group: 11,
        range: (0, 127),
    }, // 99
    MidiInstruments {
        name: "FX 5 (brightness)",
        group: 11,
        range: (0, 127),
    }, // 100
    MidiInstruments {
        name: "FX 6 (goblins)",
        group: 11,
        range: (0, 127),
    }, // 101
    MidiInstruments {
        name: "FX 7 (echoes)",
        group: 11,
        range: (0, 127),
    }, // 102
    MidiInstruments {
        name: "FX 8 (sci-fi)",
        group: 11,
        range: (0, 127),
    }, // 103
    // Ethnic
    MidiInstruments {
        name: "Sitar",
        group: 12,
        range: (48, 84),
    }, // 104
    MidiInstruments {
        name: "Banjo",
        group: 12,
        range: (48, 84),
    }, // 105
    MidiInstruments {
        name: "Shamisen",
        group: 12,
        range: (50, 79),
    }, // 106
    MidiInstruments {
        name: "Koto",
        group: 12,
        range: (55, 84),
    }, // 107
    MidiInstruments {
        name: "Kalimba",
        group: 12,
        range: (60, 84),
    }, // 108
    MidiInstruments {
        name: "Bagpipe",
        group: 12,
        range: (60, 74),
    }, // 109
    MidiInstruments {
        name: "Fiddle",
        group: 12,
        range: (55, 96),
    }, // 110
    MidiInstruments {
        name: "Shanai",
        group: 12,
        range: (60, 84),
    }, // 111
    // Percussive
    MidiInstruments {
        name: "Tinkle Bell",
        group: 13,
        range: (72, 96),
    }, // 112
    MidiInstruments {
        name: "Agogo",
        group: 13,
        range: (60, 72),
    }, // 113
    MidiInstruments {
        name: "Steel Drums",
        group: 13,
        range: (48, 72),
    }, // 114
    MidiInstruments {
        name: "Woodblock",
        group: 13,
        range: (60, 84),
    }, // 115
    MidiInstruments {
        name: "Taiko Drum",
        group: 13,
        range: (48, 60),
    }, // 116
    MidiInstruments {
        name: "Melodic Tom",
        group: 13,
        range: (43, 72),
    }, // 117
    MidiInstruments {
        name: "Synth Drum",
        group: 13,
        range: (24, 84),
    }, // 118
    MidiInstruments {
        name: "Reverse Cymbal",
        group: 13,
        range: (0, 127),
    }, // 119
    // Sound Effects
    MidiInstruments {
        name: "Guitar Fret Noise",
        group: 14,
        range: (0, 127),
    }, // 120
    MidiInstruments {
        name: "Breath Noise",
        group: 14,
        range: (0, 127),
    }, // 121
    MidiInstruments {
        name: "Seashore",
        group: 14,
        range: (0, 127),
    }, // 122
    MidiInstruments {
        name: "Bird Tweet",
        group: 14,
        range: (0, 127),
    }, // 123
    MidiInstruments {
        name: "Telephone Ring",
        group: 14,
        range: (0, 127),
    }, // 124
    MidiInstruments {
        name: "Helicopter",
        group: 14,
        range: (0, 127),
    }, // 125
    MidiInstruments {
        name: "Applause",
        group: 14,
        range: (0, 127),
    }, // 126
    MidiInstruments {
        name: "Gunshot",
        group: 14,
        range: (0, 127),
    }, // 127
];

pub static DRUM_MAP: phf::Map<u8, &'static str> = phf::phf_map! {
    35u8 => "Acoustic Bass Drum",
    36u8 => "Bass Drum 1",
    37u8 => "Side Stick",
    38u8 => "Acoustic Snare",
    39u8 => "Hand Clap",
    40u8 => "Electric Snare",
    41u8 => "Low Floor Tom",
    42u8 => "Closed Hi-Hat",
    43u8 => "High Floor Tom",
    44u8 => "Pedal Hi-Hat",
    45u8 => "Low Tom",
    46u8 => "Open Hi-Hat",
    47u8 => "Low-Mid Tom",
    48u8 => "Hi-Mid Tom",
    49u8 => "Crash Cymbal 1",
    50u8 => "High Tom",
    51u8 => "Ride Cymbal 1",
    52u8 => "Chinese Cymbal",
    53u8 => "Ride Bell",
    54u8 => "Tambourine",
    55u8 => "Splash Cymbal",
    56u8 => "Cowbell",
    57u8 => "Crash Cymbal 2",
    58u8 => "Vibraslap",
    59u8 => "Ride Cymbal 2",
    60u8 => "Hi Bongo",
    61u8 => "Low Bongo",
    62u8 => "Mute Hi Conga",
    63u8 => "Open Hi Conga",
    64u8 => "Low Conga",
    65u8 => "High Timbale",
    66u8 => "Low Timbale",
    67u8 => "High Agogo",
    68u8 => "Low Agogo",
    69u8 => "Cabasa",
    70u8 => "Maracas",
    71u8 => "Short Whistle",
    72u8 => "Long Whistle",
    73u8 => "Short Guiro",
    74u8 => "Long Guiro",
    75u8 => "Claves",
    76u8 => "Hi Wood Block",
    77u8 => "Low Wood Block",
    78u8 => "Mute Cuica",
    79u8 => "Open Cuica",
    80u8 => "Mute Triangle",
    81u8 => "Open Triangle",
};

pub static DRUM_KITS: phf::Map<u8, &'static str> = phf::phf_map! {
    0u8 => "Standard Kit",
    8u8 => "Room Kit",
    16u8 => "Power Kit",
    24u8 => "Electronic Kit",
    25u8 => "TR-808 Kit",
    32u8 => "Jazz Kit",
    40u8 => "Brush Kit",
    48u8 => "Orchestra Kit",
    56u8 => "Sound FX Kit",
};
