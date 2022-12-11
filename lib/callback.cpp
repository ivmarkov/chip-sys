#include <app/InteractionModelEngine.h>

using namespace ::chip;

extern "C" bool rustEmberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath* commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType* commandData
);

extern "C" bool rustMatterActionsPluginServerInitCallback();

bool emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath& commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType& commandData) {
    
    return rustEmberAfActionsClusterInstantActionCallback(commandObj, &commandPath, &commandData);
}

void MatterActionsPluginServerInitCallback(void) {
    rustMatterActionsPluginServerInitCallback();
}
