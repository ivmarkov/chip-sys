#pragma once

#include <platform/PlatformManager.h>

namespace singleton_raw {
    inline chip::DeviceLayer::PlatformManager* platform_mgr() {
        return &chip::DeviceLayer::PlatformMgr();
    }
}
