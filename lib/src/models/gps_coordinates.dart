/// GPS location embedded in a media file.
class GpsCoordinates {
  /// Latitude in decimal degrees. Negative values are south of the equator.
  final double lat;

  /// Longitude in decimal degrees. Negative values are west of the prime meridian.
  final double lon;

  /// Altitude in metres above sea level. Negative if below sea level.
  final double? alt;

  const GpsCoordinates({required this.lat, required this.lon, this.alt});

  @override
  String toString() => 'GpsCoordinates(lat: $lat, lon: $lon, alt: $alt)';
}
