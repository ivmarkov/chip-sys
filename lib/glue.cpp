#include <app/server/Server.h>
#include <app/util/af.h>
#include <app/util/attribute-storage.h>
#include <app/InteractionModelEngine.h>
#include <app/clusters/mode-select-server/supported-modes-manager.h>
#include <lib/core/CHIPError.h>
#include <platform/CommissionableDataProvider.h>
#include "glue.h"

using namespace ::chip;

extern "C" bool gluecb_emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath* commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType* commandData
);

extern "C" EmberAfStatus gluecb_emberAfExternalAttributeReadCallback(
    EndpointId endpoint, 
    ClusterId clusterId,
    const EmberAfAttributeMetadata* attributeMetadata, 
    uint8_t* buffer,
    uint16_t maxReadLength
);

extern "C" EmberAfStatus gluecb_emberAfExternalAttributeWriteCallback(
    EndpointId endpoint, 
    ClusterId clusterId,
    const EmberAfAttributeMetadata* attributeMetadata, 
    uint8_t* buffer
);

extern "C" bool gluecb_MatterActionsPluginServerInitCallback();

extern "C" CHIP_ERROR gluecb_CommissionableDataProvider_GetSetupDiscriminator(uint16_t* setupDiscriminator);
extern "C" CHIP_ERROR gluecb_CommissionableDataProvider_GetSpake2pIterationCount(uint32_t* iterationCount);
extern "C" CHIP_ERROR gluecb_CommissionableDataProvider_GetSpake2pSalt(MutableByteSpan* saltBuf);
extern "C" CHIP_ERROR gluecb_CommissionableDataProvider_GetSpake2pVerifier(MutableByteSpan* verifierBuf, size_t* outVerifierLen);
extern "C" CHIP_ERROR gluecb_CommissionableDataProvider_GetSetupPasscode(uint32_t* setupPasscode);

bool emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath& commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType& commandData) {
    
    return gluecb_emberAfActionsClusterInstantActionCallback(commandObj, &commandPath, &commandData);
}

EmberAfStatus emberAfExternalAttributeReadCallback(
    EndpointId endpoint, 
    ClusterId clusterId,
    const EmberAfAttributeMetadata* attributeMetadata, 
    uint8_t* buffer,
    uint16_t maxReadLength) {
    return gluecb_emberAfExternalAttributeReadCallback(endpoint, clusterId, attributeMetadata, buffer, maxReadLength);
}

EmberAfStatus emberAfExternalAttributeWriteCallback(
    EndpointId endpoint, 
    ClusterId clusterId,
    const EmberAfAttributeMetadata* attributeMetadata, 
    uint8_t* buffer) {
    return gluecb_emberAfExternalAttributeWriteCallback(endpoint, clusterId, attributeMetadata, buffer);
}

void MatterActionsPluginServerInitCallback() {
    gluecb_MatterActionsPluginServerInitCallback();
}

namespace chip {
namespace app {
namespace Clusters {
namespace ModeSelect {
const SupportedModesManager * getSupportedModesManager() {
    return NULL;
}
}
}
}
}

namespace glue {
    using namespace ::chip;

    class CommissionableDataProvider: public DeviceLayer::CommissionableDataProvider {
    public:
        CommissionableDataProvider() {}
        virtual ~CommissionableDataProvider() {}

        CHIP_ERROR GetSetupDiscriminator(uint16_t& setupDiscriminator) override {
            return gluecb_CommissionableDataProvider_GetSetupDiscriminator(&setupDiscriminator);
        }

        CHIP_ERROR SetSetupDiscriminator(uint16_t setupDiscriminator) override {
            return CHIP_ERROR_NOT_IMPLEMENTED;
        }

        CHIP_ERROR GetSpake2pIterationCount(uint32_t& iterationCount) override {
            return gluecb_CommissionableDataProvider_GetSpake2pIterationCount(&iterationCount);
        }

        CHIP_ERROR GetSpake2pSalt(MutableByteSpan& saltBuf) override {
            return gluecb_CommissionableDataProvider_GetSpake2pSalt(&saltBuf);
        }

        CHIP_ERROR GetSpake2pVerifier(MutableByteSpan& verifierBuf, size_t& outVerifierLen) override {
            return gluecb_CommissionableDataProvider_GetSpake2pVerifier(&verifierBuf, &outVerifierLen);
        }

        CHIP_ERROR GetSetupPasscode(uint32_t& setupPasscode) override {
            return gluecb_CommissionableDataProvider_GetSetupPasscode(&setupPasscode);
        }

        CHIP_ERROR SetSetupPasscode(uint32_t setupPasscode) override {
            return CHIP_ERROR_NOT_IMPLEMENTED;
        }
    };

    CommissionableDataProvider glueg_CommissionableDataProvider;
    chip::CommonCaseDeviceServerInitParams glueg_CommonCaseDeviceServerInitParams;

    void Initialize() {
        SetCommissionableDataProvider(&glueg_CommissionableDataProvider);
    }

    chip::CommonCaseDeviceServerInitParams* CommonCaseDeviceServerInitParams() {
        return &glueg_CommonCaseDeviceServerInitParams;
    }
}
