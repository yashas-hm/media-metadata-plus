Pod::Spec.new do |s|
  s.name             = 'flutter_media_metadata'
  s.version          = '0.1.0'
  s.summary          = 'Read media metadata from JPEG, HEIC, MP4, MOV, PNG and WebP.'
  s.homepage         = 'https://github.com/yourusername/flutter_media_metadata'
  s.license          = { :type => 'MIT', :file => '../LICENSE' }
  s.author           = { 'Yashas H Majmudar' => 'yashashm.dev@gmail.com' }

  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'

  s.platform         = :osx, '10.14'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
  }

  s.swift_version    = '5.0'
  s.dependency 'FlutterMacOS'

  s.resource_bundles = {'flutter_media_metadata_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
