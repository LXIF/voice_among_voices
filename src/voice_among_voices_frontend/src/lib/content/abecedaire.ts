import { tokenBuyLink } from "$lib/state/uxState";
import {get} from "svelte/store";

export const getSections = () => [
    {
        label: "A",
        title: "",
        content: "",
        children: [
            {
                title: "A to Z",
                content:
                    "This glossary explores the key philosophical, technical, and artistic ideas underlying <i>Voice Among Voices (VaV)</i>. It serves as both a reference guide and a starting point for further dialogue.",
            },
            {
                title: "Accessibility",
                content:
                    "<i>VaV</i> is accessible through any web browser. Angle 360 is available to all users free of charge and without registration. To participate, you simply need a device equipped with a microphone—such as a smartphone, laptop, or tablet. However, to contribute your own voice recordings, you must purchase one of the 359 available angles, which are offered as NFTs.",
            },
            {
                title: "Authors",
                content:
                    "<b>Dalia Donadio</b> (Co-leader and creative co-director) is a vocalist, composer, and performer whose work centers on the voice and language as universal, connecting instruments. In her solo and collaborative practices, she is exploring themes such as origin, intimacy, anatomy, and poetry, her practice spans etymology, dialects, and vocal resonance. She also integrates her roles as organizer and singing teacher into her broader artistic and research practice.<br><a class='underline' href='https://www.daliadonadio.com' target='_blank'>daliadonadio.com</a><br><br><b>Tobias Meier</b> (Co-leader and creative co-director) is an artist and musician whose works span solo performances, installations, writings, collaborations, and compositions. His collage-like pieces employ various media, while as a saxophonist he explores improvised music across different settings. Recently, he has focused on collaborative formats and group processes within creative practice, drawing from his background as a process-oriented facilitator to develop deeply democratic approaches to music-making.<br><a class='underline' href='https://www.tobias-meier.ch' target='_blank'>tobias-meier.ch</a><br><br><b>i.AM Lab, Renato Schneeberger</b> (UX/UI Design and general advice)<br><a class='underline' href='https://www.iam-lab.ch' target='_blank'>iam-lab.ch</a><br><br><b>Andri Schatz / LIFEBOD</b> (Techno-artistic consulting and translation, software engineering) is an artist, musician, designer and software engineer. On a practical level, their work is primarily focused on tying pre-lingual intuition and presence to high-tech tooling, generating deeply magickal experiences.<br><a class='underline' href='https://instagram.com/lifebod.uwu' target='_blank'>Andri Schatz Instagram</a><br><br>The first recordings to <i>VaV</i> were contributed by the following artists: tba",
            },
        ],
    },
    {
        label: "B",
        title: "",
        content: "",
        children: [
            {
                title: "Being-with",
                content:
                    "There is no being without being-with. The world permeates me as I permeate the world. This kind of being with oneself is always also being in the world. In turning toward the other – toward the world in all its ambiguity, permeability, and chaos – we come closest to ourselves.",
            },
            {
                title: "Blockchain",
                content:
                    "What makes blockchain technology compelling for <i>VaV</i> is its fundamentally collaborative nature. <i>VaV</i> is published on the ICP blockchain, allowing the work to exist as a non-local, autonomous digital <i>space</i>. A blockchain is essentially a shared digital record book that stores information across many computers simultaneously, making it impossible for any single party to control or alter the data. This technology strengthens <i>VaV</i>'s collaborative essence: rather than residing on a single server owned by one entity, the work exists as a collectively sustained space maintained by the entire network.",
            },
            {
                title: "Buy",
                content: `<i>VaV</i>-NFTs can be bought through our website or directly on <a class='underline' href='${get(tokenBuyLink)}' target='_blank'>OpenSea</a> with Ethereum. To purchase Ethereum, use your favorite trusted crypto trading platform.`,
            },
        ],
    },
    {
        label: "C",
        title: "",
        content: "",
        children: [
            {
                title: "Contact",
                content: "hello@voiceamongvoic.es",
            },
            {
                title: "Co-Creation",
                content:
                    "<i>VaV</i> can be understood as a co-creation of all participants. This means the borders between creators and listeners are dissolved, creating a dialogical shared space. This redistribution of creative responsibility raises fundamental questions about ownership, meaning, and artistic value when art becomes an ongoing process rather than a finished object. Co-creation reflects a societal shift toward participation and collective intelligence, potentially redefining both art and our understanding of community.",
            },
            {
                title: "Community",
                content:
                    "Join our <i>VaV</i>-Community on <a class='underline' href='https://oc.app' target='_blank'>OpenChat</a> and share your experiences, thoughts, and questions.",
            },
            {
                title: "Copyright & Privacy",
                content:
                    "As the holder of an NFT, you retain general copyright over your individual recording. The complete work — comprised of all 360 angles (359 + 1) – is openly licensed via <a class='underline' href='https://creativecommons.org/licenses/by-nc-nd/4.0/' target='_blank'>CC BY-NC-ND 4.0</a> (© 2025).<br><br>While your own recording can be heard publicly, it cannot be downloaded individually and is accessible only to you. You may overwrite or transfer your NFT at any time. When selling, you may choose to transfer the recording or overwrite it beforehand. Once sold, you relinquish your rights to the recording, and the new owner gains control over how it is used.",
            },
            {
                title: "Cross-Chain",
                content:
                    "This artwork leverages the existing availability and trustworthyness of BASE for its NFTs, while building its core experience on ICP. This cross-chain technology, bridging multiple blockchains fully on-chain, is enabled by recent advancements of ICP.",
            },
        ],
    },
    {
        label: "D",
        title: "",
        content: "",
        children: [
            {
                title: "Deep Democracy",
                content:
                    "In <i>VaV</i> we embrace a deeply democratic approach to all participating voices. <i>Deep Democracy</i> represents the idea that true democratic processes must go beyond majority rule to embrace all voices as sources of collective wisdom. Rather than viewing conflicts as obstacles, this concept recognizes them as essential contributions to understanding, particularly valuing marginalized perspectives. <i>Deep Democracy</i> operates on both personal and collective levels – individuals learn to acknowledge contradictory voices within themselves while communities engage in profound encounters between different worldviews. In a polarized era, this vision offers a framework for transforming differences into catalysts for richer mutual understanding and sustainable solutions.",
            },
            {
                title: "Design",
                content:
                    "The design language is minimalistic, with generous white space that naturally draws attention to the central rainbow ring and key interactive elements. The layout feels calm, deliberate, and thoughtfully crafted, reflecting the project's focus on listening, space, and gentle collaboration. The choice of a font named <i>Satoshi</i> subtly nods to the project's decentralised ethos, referencing the pseudonymous creator of Bitcoin, whose invention technology paved the way for projects like <i>VaV</i>.",
            },
            {
                title: "Duration",
                content:
                    "The playtime of <i>VaV</i> is 4 minutes. With 360 individual play-angles and therefore 360 different pieces, this results in 24 hours of sound. To fit 360 individual recordings within a circle with a radius of 2 minutes, the maximum size of each recording can be approximately 11 seconds. The maximum recording time in <i>VaV</i> is 10 seconds, leaving a minimum of one second of free space as a tribute, a sacrifice, or a gift to the community. One second to listen, reflect, and be in silence.",
            },
        ],
    },
    {
        label: "F",
        title: "",
        content: "",
        children: [
            {
                title: "Font",
                content:
                    "<i>Satoshi</i> is a modernist sans serif typeface. Its design combines typically grotesk-style letterforms, with some characters that are quite geometrically-designed. In terms of its appearance, <i>Satoshi</i> was inspired by Modernism and Industrial-Era graphic and typographic design. The family has tem weights on offer, ranging from Light to Black with complimentary italics. It is an excellent choice for use in branding, editorial, and poster design.<br><br><i>Satoshi</i> was designed by Deni Anggara for the <i>Indian Type Foundry.</i>",
            },
        ],
    },
    {
        label: "I",
        title: "",
        content: "",
        children: [
            {
                title: "Internet Computer Protocol",
                content:
                    "The Internet Computer Protocol is a next-generation Blockchain Protocol enabling full-stack decentralization. This means that the audio and physics calculations on the backend of <i>VaV</i> run fully decentralized. You can find more info under <a class='underline' href='https://internetcomputer.org' target='_blank'>internetcomputer.org</a>.",
            },
        ],
    },
    {
        label: "L",
        title: "",
        content: "",
        children: [
            {
                title: "Listening",
                content:
                    "<i>VaV</i> is about listening as an active participation in a shared acoustic space. The boundaries between listener and speaker become permeable. In the act of listening, we discover that consciousness is relational - emerging in the spaces between us, in the resonant field where voices encounter and transform each other.",
            },
        ],
    },
    {
        label: "M",
        title: "",
        content: "",
        children: [
            {
                title: "Magnetism",
                content:
                    "Each individual voice placed in the field of <i>VaV</i> influences the arrangement of all other voices. Thus, with every new contribution, the entire piece transforms. Behind this lies a physics engine that simulates magnetic repulsion. This force acts stronger when individual voices are closer together and weakens for those far apart. Magnetic repulsion symbolizes the friction between two singularities that are different from each other. This friction makes otherness tangible, and only through this can contact emerge. The aim is not for everything to be the same, because it is precisely through difference that we are with-each-other",
            },
        ],
    },
    {
        label: "N",
        title: "",
        content: "",
        children: [
            {
                title: "Non Fungible Token",
                content:
                    "Non-Fungible Tokens (NFTs) are unique digital assets stored on a blockchain, used to represent ownership of a specific item—such as an artwork, a sound recording, or another digital object. In <i>VaV</i>, NFTs assign ownership of one of the 359 available angles, enabling holders to contribute their voices and actively participate in a collective artwork.",
            },
        ],
    },
    {
        label: "O",
        title: "",
        content: "",
        children: [
            {
                title: "Open Source",
                content:
                    "The <i>VaV</i> code is open source and available <a class='underline' href='https://github.com/LXIF/voice_among_voices' target='_blank'>on github</a>.",
            },
        ],
    },
    {
        label: "P",
        title: "",
        content: "",
        children: [
            {
                title: "Participation",
                content:
                    "We invite you to participate in <i>VaV</i>: <i>At this very moment, something may catch your eye, a sound may make your eardrums vibrate, a thought may linger. Use your voice to articulate such a perception of the moment. Press Rec and sing or sigh, verbalize or hum etc. Make your voice heard among voices.</i>",
            },
            {
                title: "Price",
                content:
                    "The price is calculated as follows: y = 1 / (x * (x/25 + 0.96)), where y represents the price and x represents angles 1-360.<br>Angle 0 is not for sale and remains free to access for everyone.<br>Angle 1 is the most expensive, with prices decreasing logarithmically. This structure enables both collectibility and accessibility simultaneously.",
            },
        ],
    },
    {
        label: "Q",
        title: "",
        content: "",
        children: [
            {
                title: "Questions",
                content:
                    "Please let us know your questions via <a class='underline' href='mailto:hello@voiceamongvoices.com'>email</a> or OpenChat. Here are some of our questions:<br><br>Will there be a sense of togetherness? How much space does one voice need? How does one voice alter the perception of another? How does my voice change in the presence of others? Where do individual voices end and collective voice begin?",
            },
        ],
    },
    {
        label: "R",
        title: "",
        content: "",
        children: [
            {
                title: "Resonance",
                content:
                    "In <i>VaV</i>, voices unfold their resonance through the subtle discovery of shared vibrational planes. A deep voice can bring the overtones of a high voice into being; two voices can generate a third, initially latent tone between them. In this process, a reciprocal exchange develops between the voices: one transfers, the other absorbs, both undergo transformation through this exchange. This transmission constitutes not weakness but amplification. Thus emerge living sonic formations, interwoven through natural laws of resonance. Each voice preserves its singularity while becoming part of a larger oscillating organism.",
            },
            {
                title: "Room",
                content:
                    "digital room<br>artistic room<br>visual room<br>acoustic room<br>geometric room<br>metaphysical room<br>embodied room<br>poetic room<br><br>room for expression<br>room for encounter<br>room for co-existence<br>room for listening<br>room for silence<br>room for resonance<br>room for transformation<br>room for voices among voices",
            },
        ],
    },
    {
        label: "T",
        title: "",
        content: "",
        children: [
            {
                title: "Terms & Conditions",
                content:
                    "While <i>VaV</i> aims to be open, inclusive, and deeply democratic for everyone, there is also a more uncomfortable secondary aspect: In supporting our vision of togetherness—of being in contact while remaining different—we (the artists) curate and oversee contributions to <i>VaV</i>, removing entries that contradict this philosophy. Racist, sexist, nationalist voices, or other forms of discrimination have no place on <i>VaV</i>. While the NFTs themselves remain untouched, the corresponding recordings will be censored. For details on copyright and privacy matters, refer to <a class='underline' href='#' onclick='event.preventDefault(); window.scrollToSection(\"C\", 3)'>Copyright & Privacy</a>.",
            },
        ],
    },
    {
        label: "V",
        title: "",
        content: "",
        children: [
            {
                title: "Voice",
                content:
                    "Voice is inherent to us as humans and also connects us through its very essence. It is the instrument through which we articulate ourselves into the world and enter into resonance with others. To give voice to a moment or a feeling means to actively participate in shaping our reality. It is the act of carrying the inner outward and thereby placing ourselves in relation to others. Voice becomes the bridge between the private and the communal. To raise one's voice means to make one's own existence audible and to weave it into the fabric of all other voices. In this choir of diversity, a space of encounter emerges where each voice finds its place without drowning out the others.",
            },
            {
                title: "Vision",
                content:
                    "<i>VaV</i> is a vision of the co-existence of different voices. It sees itself as an artistic attempt to practice enduring ambiguity – a challenge we face as humans in life.<br><br>The project designs a space where different voices can exist alongside one another without their ambiguity needing to be resolved. It creates an artistic, poetic, acoustic, digital, visual and geometric space for exploring encounters.",
            },
        ],
    },
    {
        label: "Z",
        title: "",
        content: "",
        children: [
            {
                title: "Z to A",
                content:
                    "There is no one beginning or one ending. Only viewpoints, from where a single person perceives a field. A to Z is also Z to A, as both - the A and the Z - look at or listen to each other from opposite sides. They send and receive, speak and listen, sound and perceive, resonate and resonate.",
            },
        ],
    },
];
