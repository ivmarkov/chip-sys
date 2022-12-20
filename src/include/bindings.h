#include "CHIPProjectAppConfig.h"

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
#include "lib/core/DataModelTypes.h"
#include "credentials/examples/DeviceAttestationCredsExample.h"
#include "platform/ConfigurationManager.h"
#include "platform/PlatformManager.h"
#include "app/InteractionModelEngine.h"
#include "app/server/Dnssd.h"
#include "app/server/Server.h"
#include "app/server/OnboardingCodesUtil.h"
#include "app/util/af.h"
#include "app/util/attribute-storage.h"

#include "app-common/zap-generated/enums.h"
#include "app-common/zap-generated/af-structs.h"
#include "app-common/zap-generated/att-storage.h"
#include "app-common/zap-generated/attribute-id.h"
//#include "app-common/zap-generated/attribute-size.h"
#include "app-common/zap-generated/attribute-type.h"
#include "app-common/zap-generated/callback.h"
#include "app-common/zap-generated/cluster-enums.h"
#include "app-common/zap-generated/cluster-id.h"
#include "app-common/zap-generated/cluster-objects.h"
#include "app-common/zap-generated/command-id.h"

#include "glue.h"
