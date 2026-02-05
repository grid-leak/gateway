// Hardcoded kit data - all kit types, kits with their rewards, and default inventory items

use std::collections::HashMap;
use std::sync::LazyLock;

struct KitDef {
    kit_type: &'static str,
    rewards: &'static [&'static str],
}

static KITS: LazyLock<HashMap<&'static str, KitDef>> = LazyLock::new(|| {
    HashMap::from([
        // MissionKit
        (
            "2F1280F1-5D02-4F62-B1FE-C6EBFBC9FC03",
            KitDef {
                kit_type: "6C8C3FEB-2F36-483A-AADA-07F28B482CCD",
                rewards: &["1288457120"],
            },
        ), // singe-bird-wing
        (
            "5BF1FAF8-794B-47CE-AEAA-F29CC496B86C",
            KitDef {
                kit_type: "6C8C3FEB-2F36-483A-AADA-07F28B482CCD",
                rewards: &["397901330"],
            },
        ), // pigeon
        // Basic Kit Phase 1
        (
            "A9D996D9-6985-47FC-8A7D-32FE360D1082",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["1288457120"],
            },
        ), // singe-bird-wing
        (
            "7D3BF62C-C47B-4A68-9A9E-3F72AB447B93",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["397901330"],
            },
        ), // pigeon
        (
            "CDC8864D-FA35-404E-9C9A-8BDF86366E1F",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["2648777142", "3119226193"],
            },
        ), // shattered-glass-panel, Customization_Projection_ChaosLines_Red
        (
            "1EC986BD-13AD-46E1-964D-18E37070991C",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["1345791921"],
            },
        ), // Evolved_Square
        (
            "E064CDB7-2DB0-4DC0-A367-458FE0217CFE",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["2398900257"],
            },
        ), // Evolved_Frame
        (
            "56EF37EF-77D5-4AF4-88D8-72A8E03F0403",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["717478196"],
            },
        ), // ghost
        (
            "8F86B5A0-2B1A-4984-A430-1E561B09E8C9",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["24378910"],
            },
        ), // angel-wings
        (
            "60BA559A-E18D-4698-86BB-144E2BEA8DED",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["1437370938"],
            },
        ), // crystaline_07
        (
            "C46B4D54-0B69-4220-93B0-7491D5EC023A",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["2332386109"],
            },
        ), // spheres_02
        (
            "DDE96E6D-469D-487C-9117-CE1365C8ADEE",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["339938089"],
            },
        ), // grid_05
        (
            "FD478A49-8144-44C9-AC02-5833B8867D57",
            KitDef {
                kit_type: "49F6EC1E-805D-455E-B211-8228BED1C3E3",
                rewards: &["866999881"],
            },
        ), // hex_03
        // Basic Kit Phase 2
        (
            "EF3ECCBF-CBE8-4A0F-A3F1-0DAD80D6DC4A",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["3606065238"],
            },
        ), // shattered-protector-helmet
        (
            "79741B91-E054-4266-919F-B3975E2AD453",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["1143444729"],
            },
        ), // Evolved_Triforce
        (
            "D5F67426-C843-4849-85FB-E1B8572A0B11",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["224056286"],
            },
        ), // Elite_The_Hood
        (
            "2B716623-3F3F-41E2-B14C-ECA7B21E3CDF",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["3220149199"],
            },
        ), // Elite_Shinobi
        (
            "73FB7B63-82F2-448F-874E-9C1EFA86D8C0",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["795818827"],
            },
        ), // Elite_Shift
        (
            "E4ABD89A-7B4D-47DD-9229-BAF6C1A6D491",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["2190769590", "731010150"],
            },
        ), // Elite_Circle, Customization_Projection_Triangles_Yellow
        (
            "F03D6E7A-8198-4898-8B44-A0D5B19EAB8A",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["2573550572"],
            },
        ), // Elite_Nova_Prime
        (
            "00821411-68A8-47AC-A22E-7890174A2F73",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["4080907579"],
            },
        ), // butterfly
        (
            "8DC5A500-1D55-4B78-9A24-3D1F21E95938",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["4036551115", "1223661873"],
            },
        ), // spider, Customization_Projection_Rain_Red
        (
            "847B7B7A-6483-4E57-8DFC-3DB9AF94775B",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["215898171", "42501458"],
            },
        ), // key, Customization_Projection_Triangles_Orange
        (
            "269774C3-F4BF-4ABA-AD09-1BD7EEEE147E",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["2710233375"],
            },
        ), // grid_04
        (
            "7F420D89-4064-42F2-A7D1-C7E7BE6627FA",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["3868061485", "1779343174"],
            },
        ), // two-pigeons, Customization_Projection_ChaosLines_Yellow
        (
            "52BCFB04-CF5E-4EBA-9DC8-089973ECAFD7",
            KitDef {
                kit_type: "DF0241FF-5B96-403A-96D5-ED0E77FCDFD5",
                rewards: &["3331975617"],
            },
        ), // organic_04
        // Basic Kit Phase 3
        (
            "78321FCF-E4D0-4B03-9A7E-3E861A5DEDAC",
            KitDef {
                kit_type: "BB7861A9-4B9D-4ADB-8660-C9FF6C0F7986",
                rewards: &["2683930401", "3788651620"],
            },
        ), // c4, spheres_01
        (
            "BB556CF0-ACB3-4604-9045-389BF8BD1081",
            KitDef {
                kit_type: "BB7861A9-4B9D-4ADB-8660-C9FF6C0F7986",
                rewards: &["1120748012"],
            },
        ), // Evolved_Nova
        (
            "1B3249F9-FD95-4647-8AF6-FAA206DC7E1C",
            KitDef {
                kit_type: "BB7861A9-4B9D-4ADB-8660-C9FF6C0F7986",
                rewards: &["940859624", "2942898097"],
            },
        ), // open-hand-with-droplet, Customization_Projection_Streams_Red
        (
            "EC594201-154E-4CAD-9FAF-A668351000DE",
            KitDef {
                kit_type: "BB7861A9-4B9D-4ADB-8660-C9FF6C0F7986",
                rewards: &["3328861619"],
            },
        ), // delivery
        (
            "6671B289-222D-4B78-9A61-4855DA4EC381",
            KitDef {
                kit_type: "BB7861A9-4B9D-4ADB-8660-C9FF6C0F7986",
                rewards: &["347023981", "3459691569"],
            },
        ), // shattered-light-bulb, Customization_Projection_Triangles_Red
        // Basic Kit Phase 4
        (
            "06CD3E46-A1DB-4201-A574-4A7C3B5746E4",
            KitDef {
                kit_type: "38EEF404-CD00-4964-9F10-E12E9313CCB8",
                rewards: &["1396890443", "1996148657"],
            },
        ), // gift, Customization_Projection_Ghost_Red
        (
            "9EC2990A-CAB5-4BFE-828F-0D2B321F3646",
            KitDef {
                kit_type: "38EEF404-CD00-4964-9F10-E12E9313CCB8",
                rewards: &["3834883717"],
            },
        ), // Evolved_Cross
        (
            "371956A4-BEC3-492D-960B-22848A24ADF0",
            KitDef {
                kit_type: "38EEF404-CD00-4964-9F10-E12E9313CCB8",
                rewards: &["756825431"],
            },
        ), // closed-fist-revolutionary
        (
            "59400BE4-407C-48DB-BD64-5CEBA91A5295",
            KitDef {
                kit_type: "38EEF404-CD00-4964-9F10-E12E9313CCB8",
                rewards: &["420718923"],
            },
        ), // pickpocketing-hand
        (
            "408B3A94-A824-4A57-A042-2E8BE63E361D",
            KitDef {
                kit_type: "38EEF404-CD00-4964-9F10-E12E9313CCB8",
                rewards: &["70859530", "3807408409"],
            },
        ), // jacknife, Customization_Projection_ChaosLines_Green
        // Basic Kit Phase 5
        (
            "81761505-BE87-4E25-A12D-E11435025D9F",
            KitDef {
                kit_type: "C6509071-D1C5-4352-94A6-5CA46D678DE8",
                rewards: &["4201732671"],
            },
        ), // fly
        (
            "F0EAFE92-1596-42E1-A00C-B9F71D8C8006",
            KitDef {
                kit_type: "C6509071-D1C5-4352-94A6-5CA46D678DE8",
                rewards: &["1189189939", "3369933158"],
            },
        ), // Evolved_Hexagon, Customization_Projection_Rain_Yellow
        (
            "301F39BF-A027-4535-8ED3-AB293C1C20E9",
            KitDef {
                kit_type: "C6509071-D1C5-4352-94A6-5CA46D678DE8",
                rewards: &["3187831945", "1403638578"],
            },
        ), // destoyed-security-camera, Customization_Projection_ChaosLines_Orange
        // Basic Kit Phase 6
        (
            "AB1EAC40-5392-4CC5-ABFE-2CF89A7F7659",
            KitDef {
                kit_type: "CBDEFD11-5229-4444-8D36-C3B6406B874D",
                rewards: &["810778811"],
            },
        ), // shattered-turret
        (
            "5FEE722C-8608-4A0D-A258-B7AA9678A872",
            KitDef {
                kit_type: "CBDEFD11-5229-4444-8D36-C3B6406B874D",
                rewards: &["3051441250"],
            },
        ), // claw-marks
        (
            "934F383A-B437-45CB-9F66-2E481B30E41D",
            KitDef {
                kit_type: "CBDEFD11-5229-4444-8D36-C3B6406B874D",
                rewards: &["3743738366"],
            },
        ), // Evolved_Crusher
        // Basic Kit Phase 7
        (
            "F51D9463-0A60-4EA7-B40E-2D18BD0F67EA",
            KitDef {
                kit_type: "1C2F1E19-292D-49DF-8A75-0441F6C3B32C",
                rewards: &["3420869487"],
            },
        ), // cat-profile
        (
            "B6CC16FE-2958-48FC-9D92-6839BA310C2A",
            KitDef {
                kit_type: "1C2F1E19-292D-49DF-8A75-0441F6C3B32C",
                rewards: &["869446906"],
            },
        ), // shatteret-shard-building
        (
            "ADC337DB-95E4-4A7C-9728-C91DB6F6C3F5",
            KitDef {
                kit_type: "1C2F1E19-292D-49DF-8A75-0441F6C3B32C",
                rewards: &["948181069"],
            },
        ), // shatteret-padlock
        (
            "0021B2F9-CEBD-4707-A194-D6D99B29BBBC",
            KitDef {
                kit_type: "1C2F1E19-292D-49DF-8A75-0441F6C3B32C",
                rewards: &["2737174629", "162594790"],
            },
        ), // Evolved_Invincible, Customization_Projection_Streams_Yellow
        (
            "86E7CDCA-157B-4259-BBE7-F9C8B17F993F",
            KitDef {
                kit_type: "1C2F1E19-292D-49DF-8A75-0441F6C3B32C",
                rewards: &["1001049153"],
            },
        ), // Evolved_Celtic
        // Advanced Kit Phase 1
        (
            "25590F12-E99D-4275-A4D8-8FAAD5E9943F",
            KitDef {
                kit_type: "E6042F6C-AD8C-4F52-A17B-1B54EA3B38F5",
                rewards: &["3962444443", "501936717", "3769053394"],
            },
        ), // rising-sun, crystaline_08, Customization_Projection_Streams_Orange
        (
            "50B16769-CFB4-42CD-BEF4-ACE38BEC9DF6",
            KitDef {
                kit_type: "E6042F6C-AD8C-4F52-A17B-1B54EA3B38F5",
                rewards: &["1798401691", "2925611712", "1724971609"],
            },
        ), // Elite_Razor_Claw, organic_05, Customization_Projection_HalfTone_Green
        (
            "9FD1C80C-9FA6-494E-A6D1-6CAD0C8DDD91",
            KitDef {
                kit_type: "E6042F6C-AD8C-4F52-A17B-1B54EA3B38F5",
                rewards: &["3963579151", "1508639107", "2278102450"],
            },
        ), // Elite_Shield, grid_03, Customization_Projection_Scanline_Orange
        // Advanced Kit Phase (Phase 3)
        (
            "2291E3A0-AA65-4A2D-84CF-CCBBA73DF707",
            KitDef {
                kit_type: "7283A61B-470E-47D9-9622-92C4FF684CAA",
                rewards: &["3924022394", "524736417", "1112968441"],
            },
        ), // geococcyx, spheres_03, Customization_Projection_Rain_Green
        // Advanced Kit Phase 5
        (
            "B8B9099A-31AE-43E8-98A0-364220FDEBAC",
            KitDef {
                kit_type: "0BA6E906-1ABE-48E4-AEE0-C2262266C790",
                rewards: &["4161462718", "232356850", "755475321"],
            },
        ), // black-november, blocks_03, Customization_Projection_Streams_Green
        // Advanced Kit Phase 6
        (
            "4EF21635-2BC3-44C7-84D9-533F810C9675",
            KitDef {
                kit_type: "F1A67D70-66EF-44F0-AEAF-C86205B39871",
                rewards: &["1824461692", "2645211691", "870379730"],
            },
        ), // july-revolution, grid_02, Customization_Projection_Ghost_Orange
        // Advanced Kit Phase 7
        (
            "F152DC7B-C000-4819-9C88-B2E877097429",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["216500560", "1399141803", "2626703006"],
            },
        ), // che-noah, organic_02, Customization_Projection_Streams_Teal
        (
            "D4A9BD51-8E80-4535-8335-7E45F53444B9",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["1070254556", "411643776", "1402692594"],
            },
        ), // double-dragon, hex_01, Customization_Projection_HalfTone_Orange
        (
            "ABA7F111-2C64-43A1-B9DE-78F67E16E83C",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["3020191357", "2463240344", "1558888422"],
            },
        ), // Elite_Hexagon, organic_03, Customization_Projection_Ghost_Yellow
        (
            "AF9B9D32-EBED-4C92-9252-3F290C750B87",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["2863864526", "3814603228", "2532402073"],
            },
        ), // battlefield-dog-tag, crystaline_03, Customization_Projection_Scanline_Green
        (
            "223F2113-06BB-4430-991B-195D7468E0F2",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["2743303796", "4277077029", "1223506364"],
            },
        ), // leon, Evolved_Triburst, Customization_Projection_HalfTone_Blue
        (
            "6598F72A-B951-4178-AFB8-313355C26715",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["1846159064", "3804099507", "3720563836"],
            },
        ), // big-thick-book, crystaline_04, Customization_Projection_Scanline_Blue
        (
            "A41E8D29-FA20-4095-95BC-4F97ED94DB6D",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["1485773914", "3176491650", "1725571356"],
            },
        ), // rat, crystaline_05, Customization_Projection_Rain_Blue
        (
            "1348E106-5B53-4749-BC68-A314671E873A",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["2758977332", "537486201"],
            },
        ), // Evolved_Arrow, Customization_Projection_Ghost_Green
        (
            "9D18DEDF-62EB-440C-873E-BAA4840D947E",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["1758633580", "1224138686"],
            },
        ), // thumbs-up, Customization_Projection_HalfTone_Teal
        (
            "F25FAD77-2AB1-4459-A1BF-73F940CF4AB2",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["2213229352", "668878093", "3721474942"],
            },
        ), // anchor, Evolved_Hexangle, Customization_Projection_Scanline_Teal
        (
            "930DD4D3-9715-47F7-B655-022C7EF056AA",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["1956834512", "1118477620", "2625791900"],
            },
        ), // cowboyhat-and-bandana, hex_02, Customization_Projection_Streams_Blue
        (
            "D49D24FB-A726-4C5F-A903-F62921F29294",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["2387001060", "4157003967", "2501018398"],
            },
        ), // Evolved_Circle, crystaline_01, Customization_Projection_Triangles_Teal
        (
            "BCA9801E-F32E-409B-9709-0386790AB503",
            KitDef {
                kit_type: "772BCA36-59FC-46D3-81A3-5622A3C4BCD9",
                rewards: &["927967605", "3965951354", "1726482462"],
            },
        ), // mercury-god-helmet, Evolved_Shield, Customization_Projection_Rain_Teal
        // Elite Kit Phase 2
        (
            "C21288BD-ED45-49DF-862B-AE566EAB0A19",
            KitDef {
                kit_type: "D194DF7E-8C8B-4A92-9BCB-4EDD40AB0960",
                rewards: &["3333905111", "1056070740", "4149651964"],
            },
        ), // Elite_Blades, crystaline_06, Customization_Projection_ChaosLines_Blue
        // Elite Kit Phase 7
        (
            "BF6A83C1-81B2-4F89-992F-1347888783C6",
            KitDef {
                kit_type: "E3CCCE96-2FEE-46FF-A690-F6CDFAB56FEB",
                rewards: &["660883694", "1950253329", "2500107292"],
            },
        ), // Elite_Arrow, blocks_01, Customization_Projection_Triangles_Blue
        (
            "18D7396B-5D95-4A98-B08A-8CE44D4DA3D0",
            KitDef {
                kit_type: "E3CCCE96-2FEE-46FF-A690-F6CDFAB56FEB",
                rewards: &["3814728368", "3542490953", "1448742558"],
            },
        ), // mountain-peak, grid_01, Customization_Projection_Ghost_Teal
        // Preorder Kit Speed
        (
            "DA909884-CF75-4611-B87B-E6DAE82D55DC",
            KitDef {
                kit_type: "5BD6D9DE-36DF-47B4-B700-F18275BE9FBF",
                rewards: &["944773566", "3357235129", "2082001158"],
            },
        ), // cheetah, preorder_speedrunner_bg, Customization_Projection_HalfTone_Yellow
        // Preorder Kit Combat
        (
            "1345FFF3-FEB7-46E8-B776-82A585466098",
            KitDef {
                kit_type: "3FB530DF-E8EF-4AB5-BDAC-7E11CA2E0E6D",
                rewards: &["2364205284", "4189180625", "1544417233"],
            },
        ), // fist, preorder_combat_bg, Customization_Projection_Scanline_Red
        // Engagement Kit
        (
            "A8B5F5F6-0A32-4FBB-93EC-654BCD6DE806",
            KitDef {
                kit_type: "0831C039-1EC2-49DE-96BA-8A663F5D268E",
                rewards: &["4093061924"],
            },
        ), // shattered-oldschool-footprint
        // Beta Kit
        (
            "C5B8CEF2-66A7-4119-BBBA-1A9E712C797F",
            KitDef {
                kit_type: "8C7D0548-24DA-4105-A214-945FC7A45A1A",
                rewards: &["1823346219"],
            },
        ), // stallion
        // Re-engagement Kit 1
        (
            "2FD08BA5-D355-4DE1-8C5D-91036D4F66FE",
            KitDef {
                kit_type: "7B8F24AF-F3E4-4189-8B7D-D37BF0D112DA",
                rewards: &["3089268753"],
            },
        ), // ikarus_portrait
        // Re-engagement Kit 2
        (
            "0C163F9B-345E-46AE-A2DA-2D6444157C7D",
            KitDef {
                kit_type: "392C6D30-5130-464F-9465-EF4DD1E460CE",
                rewards: &["2361194228"],
            },
        ), // feather
        // Re-engagement Kit 3
        (
            "42A98283-325A-4C91-AE7F-40AA3FC07683",
            KitDef {
                kit_type: "83B55549-5970-4750-B033-0D12925EA8D6",
                rewards: &["3784620829"],
            },
        ), // bee
    ])
});

/// Default inventory items that all users start with
const DEFAULT_ITEMS: &[&str] = &[
    "1183558251", // spheres_04
    "2046797545", // organic_01
    "2556762952", // crystaline_02
    "2973486374", // blocks_02
    "1514008114", // bop-chaser
    "3049936381", // Basic_Arrow
    "769109517",  // Basic_Circle
    "3174165210", // Basic_Diamond
    "4096073036", // Basic_Hexagon
    "294373848",  // Basic_Octagon
    "1679314630", // Basic_Pentagon
    "1311986746", // Basic_Shield
    "996320292",  // Basic_Square
    "3732465449", // Basic_Star
    "54308782",   // Basic_Triangle
    "244578012",  // Customization_Projection_RunVis
];

pub fn get_kit_type(kit_id: &str) -> Option<&'static str> {
    let normalized = kit_id.to_uppercase();
    KITS.get(normalized.as_str()).map(|def| def.kit_type)
}

pub fn get_kit_rewards(kit_id: &str) -> Option<&'static [&'static str]> {
    let normalized = kit_id.to_uppercase();
    KITS.get(normalized.as_str()).map(|def| def.rewards)
}

pub fn get_default_items() -> &'static [&'static str] {
    DEFAULT_ITEMS
}
