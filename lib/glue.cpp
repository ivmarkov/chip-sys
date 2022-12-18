#include <app/util/af.h>
#include <app/util/attribute-storage.h>
#include <app/InteractionModelEngine.h>
#include <lib/core/CHIPError.h>
#include <platform/CommissionableDataProvider.h>

using namespace ::chip;

extern "C" bool gluecb_emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath* commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType* commandData
);

extern "C" bool gluecb_MatterActionsPluginServerInitCallback();

extern "C" uint16_t gluecb_CommissionableDataProvider_GetSetupDiscriminator(uint16_t* setupDiscriminator);
extern "C" uint16_t gluecb_CommissionableDataProvider_GetSpake2pIterationCount(uint32_t* iterationCount);
extern "C" uint16_t gluecb_CommissionableDataProvider_GetSpake2pSalt(MutableByteSpan* saltBuf);
extern "C" uint16_t gluecb_CommissionableDataProvider_GetSpake2pVerifier(MutableByteSpan* verifierBuf, size_t* outVerifierLen);
extern "C" uint16_t gluecb_CommissionableDataProvider_GetSetupPasscode(uint32_t* setupPasscode);

bool emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath& commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType& commandData) {
    
    return gluecb_emberAfActionsClusterInstantActionCallback(commandObj, &commandPath, &commandData);
}

void MatterActionsPluginServerInitCallback(void) {
    gluecb_MatterActionsPluginServerInitCallback();
}

class CommissionableDataProvider: public DeviceLayer::CommissionableDataProvider {
public:
    CommissionableDataProvider() {}
    virtual ~CommissionableDataProvider() {}

    CHIP_ERROR GetSetupDiscriminator(uint16_t& setupDiscriminator) override {
        return ChipError(gluecb_CommissionableDataProvider_GetSetupDiscriminator(&setupDiscriminator));
    }

    CHIP_ERROR SetSetupDiscriminator(uint16_t setupDiscriminator) override {
        return CHIP_ERROR_NOT_IMPLEMENTED;
    }

    CHIP_ERROR GetSpake2pIterationCount(uint32_t& iterationCount) override {
        return ChipError(gluecb_CommissionableDataProvider_GetSpake2pIterationCount(&iterationCount));
    }

    CHIP_ERROR GetSpake2pSalt(MutableByteSpan& saltBuf) override {
        return ChipError(gluecb_CommissionableDataProvider_GetSpake2pSalt(&saltBuf));
    }

    CHIP_ERROR GetSpake2pVerifier(MutableByteSpan& verifierBuf, size_t& outVerifierLen) override {
        return ChipError(gluecb_CommissionableDataProvider_GetSpake2pVerifier(&verifierBuf, &outVerifierLen));
    }

    CHIP_ERROR GetSetupPasscode(uint32_t& setupPasscode) override {
        return ChipError(gluecb_CommissionableDataProvider_GetSetupPasscode(&setupPasscode));
    }

    CHIP_ERROR SetSetupPasscode(uint32_t setupPasscode) override {
        return CHIP_ERROR_NOT_IMPLEMENTED;
    }
};

CommissionableDataProvider glue_CommissionableDataProvider;

extern "C" void glue_InitCommissionableDataProvider() {
    SetCommissionableDataProvider(&glue_CommissionableDataProvider);
}

extern "C" uint32_t glue_emberAfSetDeviceTypeList(EndpointId endpoint, const EmberAfDeviceType* deviceTypeList, size_t deviceTypeListLen) {
    return emberAfSetDeviceTypeList(endpoint, chip::Span<const EmberAfDeviceType>(deviceTypeList, deviceTypeListLen)).AsInteger();
}

extern "C" EmberAfStatus glue_emberAfSetDynamicEndpoint(
    uint16_t index, 
    chip::EndpointId id, 
    const EmberAfEndpointType* ep,
    const chip::DataVersion* dataVersionStorage,
    size_t dataVersionStorageLen,
    const EmberAfDeviceType* deviceTypeList,
    size_t deviceTypeListLen,
    chip::EndpointId parentEndpointId) {
    return emberAfSetDynamicEndpoint(
        index, 
        id, 
        ep, 
        chip::Span<const chip::DataVersion>(dataVersionStorage, dataVersionStorageLen),
        chip::span<const EmberAfDeviceType>(deviceTypeList, deviceTypeListLen),
        parentEndpointId);
}

extern "C" uint8_t* glue_MutableByteSpan_data(MutableByteSpan* span) {
    return span->data();
}

extern "C" size_t glue_MutableByteSpan_size(MutableByteSpan* span) {
    return span->size();
}

extern "C" void glue_MutableByteSpan_reduce_size(MutableByteSpan* span, size_t size) {
    span->reduce_size(size);
}
