# Intention

The idea is to understand deeply network standards for multimedia delivery and playback.
The most well-known candidate is an RTP. Of course, RTP requires negotiation, so
SDP and SIP are also should be understood.

The first steps is to write data-structures to map domain definitions into the code, while preserving relations between them.

## Parsers

It is desired to have composable parsers in terms of what exact structure application wants to get.
So it means that some parsers should be related to each others (i.e. parsing of dependent headers).

