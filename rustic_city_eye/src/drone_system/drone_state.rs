///  El Drone puede tener distintos estados:
/// - Waiting: esta circulando en su radio de operacion, pero no esta atendiendo ningun incidente.
/// - AttendingIncident: un nuevo incidente fue cargado por la app de monitoreo, y el Drone fue asignado
///                         a resolverlo.
/// - LowBatteryLevel: el Drone se quedo sin bateria, por lo que va a su central a cargarse, y no va a volver a
///                    funcionar hasta que tenga el nivel de bateria completo(al terminar de cargarse, vuelve a
///                    tener el estado Waiting).
#[derive(Debug, PartialEq)]
pub enum DroneState {
    Waiting,
    AttendingIncident,
    LowBatteryLevel,
}
