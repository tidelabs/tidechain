window.SIDEBAR_ITEMS = {"enum":[["GossipsubEvent","Event that can be emitted by the gossipsub behaviour."],["MessageAcceptance","Validation kinds from the application for received messages."],["MessageAuthenticity","Determines if published messages should be signed or not."],["ValidationMode","The types of message validation that can be employed by gossipsub."]],"fn":[["score_parameter_decay","Computes the decay factor for a parameter, assuming the `decay_interval` is 1s and that the value decays to zero if it drops below 0.01."],["score_parameter_decay_with_base","Computes the decay factor for a parameter using base as the `decay_interval`."]],"mod":[["error","Error types that can result from gossipsub."],["metrics","A set of metrics used to help track and diagnose the network behaviour of the gossipsub protocol."],["protocol",""],["subscription_filter",""],["time_cache","This implements a time-based LRU cache for checking gossipsub message duplicates."]],"struct":[["FastMessageId",""],["Gossipsub","Network behaviour that handles the gossipsub protocol."],["GossipsubConfig","Configuration parameters that define the performance of the gossipsub network."],["GossipsubConfigBuilder","The builder struct for constructing a gossipsub configuration."],["GossipsubMessage","The message sent to the user after a [`RawGossipsubMessage`] has been transformed by a [`crate::DataTransform`]."],["GossipsubRpc","An RPC received/sent."],["IdentityTransform","The default transform, the raw data is propagated as is to the application layer gossipsub."],["MessageId",""],["PeerScoreParams",""],["PeerScoreThresholds",""],["RawGossipsubMessage","A message received by the gossipsub system and stored locally in caches.."],["Topic","A gossipsub topic."],["TopicHash",""],["TopicScoreParams",""]],"trait":[["DataTransform","A general trait of transforming a [`RawGossipsubMessage`] into a [`GossipsubMessage`]. The [`RawGossipsubMessage`] is obtained from the wire and the [`GossipsubMessage`] is used to calculate the [`crate::MessageId`] of the message and is what is sent to the application."],["Hasher","A generic trait that can be extended for various hashing types for a topic."]],"type":[["IdentTopic",""],["Sha256Topic",""]]};