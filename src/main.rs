mod rtp {
    use std::collections::{
        HashSet,
    };
    // TODO: define big-endian byte order
    // use ByteOrder = core::byte_order::BigEndian;

    #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
    pub struct SynchronizationSource(u32);

    #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
    pub struct ContributingSource(u32);

    impl Into<SynchronizationSource> for ContributingSource {
        fn into(self: Self) -> SynchronizationSource {
            SynchronizationSource(self.0)
        }
    }

    impl Into<ContributingSource> for SynchronizationSource {
        fn into(self: Self) -> ContributingSource {
            ContributingSource(self.0)
        }
    }

    // Q: is an order matters in source list?
    //    i.e. should loudest speaker be the first item
    //    in the list or this is out of scope of csrc list
    //    meaning?
    pub type ContributingSourceList = Vec<ContributingSource>;

    #[derive(Default)]
    pub struct Session {
        sources: HashSet<SynchronizationSource>
    }

    impl Session {

        pub fn new() -> Self {
            Default::default()
        }

        pub fn add_new_source(&mut self) -> SynchronizationSource {
            loop {
                let source_candidate = SynchronizationSource(rand::random());

                // TODO: implement circuit-breaker to prevent
                //       infinite retries if the sources list
                //       is full
                if self.sources.insert(source_candidate) {
                    return source_candidate;
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum Version {
        // The value 0 is used by the protocol initially implemented in the "vat" audio tool.)
        ZERO,
        // The value 1 is used by the first draft version of RTP (even before [RFC1889]
        ONE,
        // The version defined by this specification [RFC3550] is two (2)
        TWO,
        RESERVED
    }

    /// From [RFC3550]
    ///
    /// The RTP header has the following format:
    ///  0                   1                   2                   3
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |V=2|P|X|  CC   |M|     PT      |       sequence number         |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |                           timestamp                           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |           synchronization source (SSRC) identifier            |
    /// +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
    /// |            contributing source (CSRC) identifiers             |
    /// |                             ....                              |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    pub struct Header {
        // version (V): 2 bits
        //
        // This field identifies the version of RTP.  The version defined by
        // this specification is two (2).  (The value 1 is used by the first
        // draft version of RTP and the value 0 is used by the protocol
        // initially implemented in the "vat" audio tool.)
        version: Version,

        // padding (P): 1 bit
        //
        // If the padding bit is set, the packet contains one or more
        // additional padding octets at the end which are not part of the
        // payload.  The last octet of the padding contains a count of how
        // many padding octets should be ignored, including itself.  Padding
        // may be needed by some encryption algorithms with fixed block sizes
        // or for carrying several RTP packets in a lower-layer protocol data
        // unit.
        padding: bool,

        // extension (X): 1 bit
        //
        // If the extension bit is set, the fixed header MUST be followed by
        // exactly one header extension, with a format defined in Section
        // 5.3.1 [RFC3550].
        extension: bool,

        // CSRC count (CC): 4 bits
        //
        // The CSRC count contains the number of CSRC identifiers that follow
        // the fixed header.
        csrc_count: u8,

        // marker (M): 1 bit
        //
        // The interpretation of the marker is defined by a profile.  It is
        // intended to allow significant events such as frame boundaries to
        // be marked in the packet stream.  A profile MAY define additional
        // marker bits or specify that there is no marker bit by changing the
        // number of bits in the payload type field (see Section 5.3).
        marker: bool,

        // payload type (PT): 7 bits
        //
        // This field identifies the format of the RTP payload and determines
        // its interpretation by the application.  A profile MAY specify a
        // default static mapping of payload type codes to payload formats.
        // Additional payload type codes MAY be defined dynamically through
        // non-RTP means (see Section 3).  A set of default mappings for
        // audio and video is specified in the companion RFC 3551 [1].  An
        // RTP source MAY change the payload type during a session, but this
        // field SHOULD NOT be used for multiplexing separate media streams
        // (see Section 5.2).
        // A receiver MUST ignore packets with payload types that it does not
        // understand.
        //
        // Even though `payload_type` is an 7bits field in the bytes
        // it is mpre friendly for Rust programming to have `u8` variable
        payload_type: u8,

        // sequence number: 16 bits
        //
        // The sequence number increments by one for each RTP data packet
        // sent, and may be used by the receiver to detect packet loss and to
        // restore packet sequence.  The initial value of the sequence number
        // SHOULD be random (unpredictable) to make known-plaintext attacks
        // on encryption more difficult, even if the source itself does not
        // encrypt according to the method in Section 9.1, because the
        // packets may flow through a translator that does.  Techniques for
        // choosing unpredictable numbers are discussed in [17].
        sequence_number: u16,

        // timestamp: 32 bits
        //
        // The timestamp reflects the sampling instant of the first octet in
        // the RTP data packet.  The sampling instant MUST be derived from a
        // clock that increments monotonically and linearly in time to allow
        // synchronization and jitter calculations (see Section 6.4.1).  The
        // resolution of the clock MUST be sufficient for the desired
        // synchronization accuracy and for measuring packet arrival jitter
        // (one tick per video frame is typically not sufficient).  The clock
        // frequency is dependent on the format of data carried as payload
        // and is specified statically in the profile or payload format
        // specification that defines the format, or MAY be specified
        // dynamically for payload formats defined through non-RTP means.  If
        // RTP packets are generated periodically, the nominal sampling
        // instant as determined from the sampling clock is to be used, not a
        // reading of the system clock.  As an example, for fixed-rate audio
        // the timestamp clock would likely increment by one for each
        // sampling period.  If an audio application reads blocks covering

        // 160 sampling periods from the input device, the timestamp would be
        // increased by 160 for each such block, regardless of whether the
        // block is transmitted in a packet or dropped as silent.

        // The initial value of the timestamp SHOULD be random, as for the
        // sequence number.  Several consecutive RTP packets will have equal
        // timestamps if they are (logically) generated at once, e.g., belong
        // to the same video frame.  Consecutive RTP packets MAY contain
        // timestamps that are not monotonic if the data is not transmitted
        // in the order it was sampled, as in the case of MPEG interpolated
        // video frames.  (The sequence numbers of the packets as transmitted
        // will still be monotonic.)

        // RTP timestamps from different media streams may advance at
        // different rates and usually have independent, random offsets.
        // Therefore, although these timestamps are sufficient to reconstruct
        // the timing of a single stream, directly comparing RTP timestamps
        // from different media is not effective for synchronization.
        // Instead, for each medium the RTP timestamp is related to the
        // sampling instant by pairing it with a timestamp from a reference
        // clock (wallclock) that represents the time when the data
        // corresponding to the RTP timestamp was sampled.  The reference
        // clock is shared by all media to be synchronized.  The timestamp
        // pairs are not transmitted in every data packet, but at a lower
        // rate in RTCP SR packets as described in Section 6.4.

        // The sampling instant is chosen as the point of reference for the
        // RTP timestamp because it is known to the transmitting endpoint and
        // has a common definition for all media, independent of encoding
        // delays or other processing.  The purpose is to allow synchronized
        // presentation of all media sampled at the same time.

        // Applications transmitting stored data rather than data sampled in
        // real time typically use a virtual presentation timeline derived
        // from wallclock time to determine when the next frame or other unit
        // of each medium in the stored data should be presented.  In this
        // case, the RTP timestamp would reflect the presentation time for
        // each unit.  That is, the RTP timestamp for each unit would be
        // related to the wallclock time at which the unit becomes current on
        // the virtual presentation timeline.  Actual presentation occurs
        // some time later as determined by the receiver.

        // An example describing live audio narration of prerecorded video
        // illustrates the significance of choosing the sampling instant as
        // the reference point.  In this scenario, the video would be
        // presented locally for the narrator to view and would be
        // simultaneously transmitted using RTP.  The "sampling instant" of a
        // video frame transmitted in RTP would be established by referencing

        // its timestamp to the wallclock time when that video frame was
        // presented to the narrator.  The sampling instant for the audio RTP
        // packets containing the narrator's speech would be established by
        // referencing the same wallclock time when the audio was sampled.
        // The audio and video may even be transmitted by different hosts if
        // the reference clocks on the two hosts are synchronized by some
        // means such as NTP.  A receiver can then synchronize presentation
        // of the audio and video packets by relating their RTP timestamps
        // using the timestamp pairs in RTCP SR packets.
        timestamp: u32,

        // SSRC: 32 bits
        //
        // The SSRC field identifies the synchronization source.  This
        // identifier SHOULD be chosen randomly, with the intent that no two
        // synchronization sources within the same RTP session will have the
        // same SSRC identifier.  An example algorithm for generating a
        // random identifier is presented in Appendix A.6.  Although the
        // probability of multiple sources choosing the same identifier is
        // low, all RTP implementations must be prepared to detect and
        // resolve collisions.  Section 8 describes the probability of
        // collision along with a mechanism for resolving collisions and
        // detecting RTP-level forwarding loops based on the uniqueness of
        // the SSRC identifier.  If a source changes its source transport
        // address, it must also choose a new SSRC identifier to avoid being
        // interpreted as a looped source (see Section 8.2).
        ssrc: SynchronizationSource,

        // CSRC list: 0 to 15 items, 32 bits each
        // The CSRC list identifies the contributing sources for the payload
        // contained in this packet.  The number of identifiers is given by
        // the CC field.  If there are more than 15 contributing sources,
        // only 15 can be identified.  CSRC identifiers are inserted by
        // mixers (see Section 7.1), using the SSRC identifiers of
        // contributing sources.  For example, for audio packets the SSRC
        // identifiers of all sources that were mixed together to create a
        // packet are listed, allowing correct talker indication at the
        // receiver.
        csrc_list: [ContributingSource; 15],

        _extension: Option<HeaderExtension<()>>,
    }

    // I start thinking that there are two representations needed
    // for the users of the protocol:
    // 1. Parsed from the bytes
    // 2. Newly created packet header (without even payload)
    struct HeaderExtension <T> {
        defined_by_profile: u16,
        length: u16,
        value: T
    }
}

fn main() {
    println!("Hello, MoIP!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forming_contributing_source_list() {
        let mut rtp_session = rtp::Session::new();
        let microphone_source = rtp_session.add_new_source();
        let external_card_source = rtp_session.add_new_source();

        let mut mixer_sources = rtp::ContributingSourceList::default();

        mixer_sources.push(microphone_source.into());
        mixer_sources.push(external_card_source.into());
    }
}
