# Service Connector

Service Connector acts a signalling server for nodes in the network. Service Connector is required to create the self-healing, highly available, versitial mesh network with the role of connecting nodes to one another, handling disconnections, healing properties, etc. These healing properties work as such: if the service connector were to go down, the cluster lives on and will reconnect to the connector, indicating the connectors death, which will propagate the Node SnapShot again.
