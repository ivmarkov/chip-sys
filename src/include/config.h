//#pragma once

#define CHIP_HAVE_CONFIG_H 1

#define CHIP_MINMDNS_USE_EPHEMERAL_UNICAST_PORT 1
#define CHIP_MINMDNS_HIGH_VERBOSITY 0
#define CHIP_MINMDNS_DEFAULT_POLICY 1
#define CHIP_ADDRESS_RESOLVE_IMPL_INCLUDE_HEADER <lib/address_resolve/AddressResolve_DefaultImpl.h>

#ifdef NDEBUG
#define CONFIG_IS_DEBUG 0
#else
#define CONFIG_IS_DEBUG 1
#endif
