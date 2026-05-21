class GpsCoordinates {
  final double lat;
  final double lon;
  final double? alt;

  const GpsCoordinates({required this.lat, required this.lon, this.alt});

  @override
  String toString() => 'GpsCoordinates(lat: $lat, lon: $lon, alt: $alt)';
}