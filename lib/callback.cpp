#include <app/InteractionModelEngine.h>

using namespace ::chip;

extern "C" bool rustEmberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath* commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType* commandData
);

bool emberAfActionsClusterInstantActionCallback(
    app::CommandHandler* commandObj, 
    const app::ConcreteCommandPath& commandPath,
    const app::Clusters::Actions::Commands::InstantAction::DecodableType& commandData) {
    
    return rustEmberAfActionsClusterInstantActionCallback(commandObj, &commandPath, &commandData);
}
