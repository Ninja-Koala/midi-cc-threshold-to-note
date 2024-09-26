use lv2::prelude::*;
use wmidi::*;

#[derive(PortCollection)]
pub struct Ports {
    input: InputPort<AtomPort>,
    output: OutputPort<AtomPort>,
}

#[derive(FeatureCollection)]
pub struct Features<'a> {
    map: LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs {
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[uri("https://github.com/Ninja-Koala/midi-cc-threshold-to-note")]
pub struct Midithreshold {
	note_active: bool,
	threshold: U7,
	note_value: Note,
	note_on_velocity: U7,
	note_off_velocity: U7,
    cc_parameter: U7,
	input_channel: Channel,
	output_channel: Channel,
    urids: URIDs,
}

impl Plugin for Midithreshold {
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        Some(Self {
            note_active: false,
			threshold: 64.try_into().unwrap(),
            note_value: Note::C2,
			note_on_velocity: 127.try_into().unwrap(),
			note_off_velocity: 127.try_into().unwrap(),
            cc_parameter: 0.try_into().unwrap(),
			input_channel: Channel::Ch1,
			output_channel: Channel::Ch1,
            urids: features.map.populate_collection()?,
        })
    }

    fn run(&mut self, ports: &mut Ports, _: &mut (), _: u32) {
        let input_sequence = ports
            .input
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        let mut output_sequence = ports
            .output
            .init(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.unit.frame),
            )
            .unwrap();

        for (timestamp, atom) in input_sequence {

            let message = if let Some(message) = atom.read(self.urids.midi.wmidi, ()) {
                message
            } else {
                continue;
            };

            match message {
                MidiMessage::ControlChange(channel, number, value) => {
					if channel == self.input_channel &&
					number == self.cc_parameter {
						if value < self.threshold && self.note_active {
							self.note_active = false;
							output_sequence
								.init(
									timestamp,
									self.urids.midi.wmidi,
									MidiMessage::NoteOff(self.output_channel.into(), self.note_value.into(), self.note_off_velocity.into())
								)
								.unwrap();
						} else if value >= self.threshold && !self.note_active {
							self.note_active = true;
							output_sequence
								.init(
									timestamp,
									self.urids.midi.wmidi,
									MidiMessage::NoteOn(self.output_channel, self.note_value, self.note_on_velocity)
								)
								.unwrap();
						}
					}
                }
                _ => (),
            }
        }
    }

	// not sure if i want this
    fn activate(&mut self, _features: &mut Features<'static>) {
        self.note_active = false;
    }
}

lv2_descriptors!(Midithreshold);
