use autocxx::prelude::*; // use all the main autocxx functions

include_cpp! {
    #include "util.h"
    //#include "ChipProjectConfig.h"
    #include "platform/PlatformManager.h"

    safety!(unsafe)
    generate!("chip::DeviceLayer::PlatformManager")
}
