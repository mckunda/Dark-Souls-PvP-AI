//used in handler
pub const AutoRedSign: bool = true;
pub const DisableAi: bool = false;
pub const TrainNeuralNet: bool = false;

//used in initalizeFann
pub const TrainAttackNet: i32 = 0;
pub const TrainBackstabNet: i32 = 1;
pub const FeedNeuralNet: bool = false;

//*****NOTE****: if this is longer than ~120 characters FANN will crash when trying to open the .net file. Try not to do that????? Sorry
pub const NeuralNetFolderLocation: &'static str = "E:/Code Workspace/Dark Souls AI C/Neural Nets";

//used in helper utils (for camera)
pub const OolicelMap: i32 = 1;

pub const BackstabMetaOnly: i32 = 0;

//used in gui
pub const ENABLEGUI: i32 = 0;
pub const ENABLEPRINT: i32 = 0;
pub const REDIRECTTOFILE: i32 = 0; //WARNING: produces 1GB every 2 min
pub const PORT: i32 = 4149;
