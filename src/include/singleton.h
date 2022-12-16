#pragma once

#include <app/server/Server.h>
#include <platform/ConfigurationManager.h>
#include <platform/PlatformManager.h>

namespace singleton_raw {
    inline chip::Server* server() {
        return &chip::Server::GetInstance();
    }

    inline chip::DeviceLayer::PlatformManager* platform_mgr() {
        return &chip::DeviceLayer::PlatformMgr();
    }

    inline chip::DeviceLayer::ConfigurationManager* configuration_mgr() {
        return &chip::DeviceLayer::ConfigurationMgr();
    }
}
