Pod::Spec.new do |s|
  s.name             = 'media_metadata_plus'
  s.version          = '1.2.1'
  s.summary          = 'Read media metadata from JPEG, HEIC, MP4, MOV, PNG and WebP.'
  s.homepage         = 'https://github.com/yashas-hm/media-metadata-plus'
  s.license          = { :type => 'MIT', :file => '../LICENSE' }
  s.author           = { 'Yashas H Majmudar' => 'yashashm.dev@gmail.com' }

  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'

  s.platform         = :osx, '10.14'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
  }

  s.swift_version        = '5.0'
  s.dependency           'FlutterMacOS'
  s.vendored_frameworks  = 'Frameworks/media_metadata_plus.xcframework'

  s.resource_bundles = {'media_metadata_plus_privacy' => ['Resources/PrivacyInfo.xcprivacy']}
end
