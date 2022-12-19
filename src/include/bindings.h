#include <system/SystemBuildConfig.h>
#include <platform/CHIPDeviceBuildConfig.h>

#include CHIP_PROJECT_CONFIG_INCLUDE
#include CHIP_PLATFORM_CONFIG_INCLUDE
#include SYSTEM_PROJECT_CONFIG_INCLUDE
#include SYSTEM_PLATFORM_CONFIG_INCLUDE
#include CHIP_SYSTEM_LAYER_IMPL_CONFIG_FILE
#include CHIP_DEVICE_PLATFORM_CONFIG_INCLUDE

#ifdef CHIP_SYSTEM_CONFIG_USE_SOCKETS
#define INET_TCP_END_POINT_IMPL_CONFIG_FILE <inet/TCPEndPointImplSockets.h>
#define INET_UDP_END_POINT_IMPL_CONFIG_FILE <inet/UDPEndPointImplSockets.h>
#endif

#define CHIP_ADDRESS_RESOLVE_IMPL_INCLUDE_HEADER <lib/address_resolve/AddressResolve_DefaultImpl.h>

#include "lib/core/CHIPError.h"
#include "credentials/examples/DeviceAttestationCredsExample.h"
#include "platform/ConfigurationManager.h"
#include "platform/PlatformManager.h"
#include "app/InteractionModelEngine.h"
#include "app/server/Dnssd.h"
#include "app/server/Server.h"
#include "app/server/OnboardingCodesUtil.h"
#include "app/util/af.h"
#include "app/util/attribute-storage.h"
#include "glue.h"

// overrides CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT in CHIPProjectConfig
//#define CHIP_DEVICE_CONFIG_DYNAMIC_ENDPOINT_COUNT 16
