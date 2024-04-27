#ifndef BRIDGE_SENDER_H
#define BRIDGE_SENDER_H

#include <iostream>
#include <memory>

#include "rust/cxx.h"

struct BridgeSender;
using BridgeSenderSharedPtr = std::shared_ptr<rust::Box<BridgeSender>>;

struct BridgeSendEndNotifier;

struct MeshData;

#endif
