use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON,
    "/home/wormphlegm/Projects/my_rl/resources/SmallDungeon_80x50.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "/home/wormphlegm/Projects/my_rl/resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE2, "/home/wormphlegm/Projects/my_rl/resources/wfc-demo2.xp");

pub struct RexAssets {
    pub menu : XpFile
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON,
            "/home/wormphlegm/Projects/my_rl/resources/SmallDungeon_80x50.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "/home/wormphlegm/Projects/my_rl/resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE2, "/home/wormphlegm/Projects/my_rl/resources/wfc-demo2.xp");

        RexAssets{
            menu : XpFile::from_resource(
                       "/home/wormphlegm/Projects/my_rl/resources/SmallDungeon_80x50.xp").unwrap()
        }
    }
}
