#ifndef BRIDGE_SENDER_H
#define BRIDGE_SENDER_H

#include <iostream>
#include <memory>

#include "rust/cxx.h"

struct BridgeSender;
typedef std::shared_ptr<rust::Box<BridgeSender>> BridgeSenderSharedPtr;

#endif
