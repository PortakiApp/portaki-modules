Release candidate for Portaki modules
---


<details><summary>access-guide: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/access-guide-v1.0.0...access-guide-v1.1.0) (2026-09-29)


### Features

* **access-guide:** add catalogue listing ([2b27528](https://github.com/PortakiApp/portaki-modules/commit/2b27528290a0f853726e60977c78fbd0619a868a))
* **access-guide:** allow stay-link emailContext ([6d210d0](https://github.com/PortakiApp/portaki-modules/commit/6d210d0a191c28c4050f1c98d33922eb9d2388c3))
* **access-guide:** check entry code before publish ([3eae845](https://github.com/PortakiApp/portaki-modules/commit/3eae84570cde22224bf72b747c0ac58dc28f045f))
* **access-guide:** declare config, codes secret ([dd2294f](https://github.com/PortakiApp/portaki-modules/commit/dd2294f784efc7a9e2d72295df2d2ab38f72a5d9))
* **access-guide:** dispatch examples, scenario tests ([f8efcfa](https://github.com/PortakiApp/portaki-modules/commit/f8efcfa21c9ee4f97c53d3ab1ac9acb92c05c3b5))
* **access-guide:** email guests on code change ([66be772](https://github.com/PortakiApp/portaki-modules/commit/66be7723ef8e477ee38bb0643d22d8e319cc1876))
* **access-guide:** label property map marker ([82f5e5b](https://github.com/PortakiApp/portaki-modules/commit/82f5e5b102c48df789bdf4629cf2469702f2b9ec))
* **access-guide:** release host SDUI as 0.1.1 ([b95fc21](https://github.com/PortakiApp/portaki-modules/commit/b95fc21f3cf9967abe6d3c919374919de3ec490e))
* **access-guide:** translate guest texts ([1547614](https://github.com/PortakiApp/portaki-modules/commit/1547614a2cc3839e95e9823a243c8cec15750bc5))
* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **email:** localize module guest emails ([d7186a0](https://github.com/PortakiApp/portaki-modules/commit/d7186a0b21941d476d12d9a6c14e3965f5c4d057))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** open cards via fullscreen overlay ([530aa1e](https://github.com/PortakiApp/portaki-modules/commit/530aa1e3670d88e906255199bcf736613b9e28c8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** add emailContext guest queries ([5d8baeb](https://github.com/PortakiApp/portaki-modules/commit/5d8baeb2f5098bb7eb247620273368827e737d1d))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** emit code-changed and review submitted ([7657a9f](https://github.com/PortakiApp/portaki-modules/commit/7657a9fb036ee218698be08f5bd4486cd2382d0f))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** own mail and lost-found host SDUI ([a62e5be](https://github.com/PortakiApp/portaki-modules/commit/a62e5be10fa3049690cdb6185975d7175933e83e))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **access-guide:** align RevealPolicy wire names ([ecf6255](https://github.com/PortakiApp/portaki-modules/commit/ecf6255c79d3ab66c663b7a6c6abda4479875da3))
* **access-guide:** block publish until method is set ([d561409](https://github.com/PortakiApp/portaki-modules/commit/d5614097e2d27c9c2a0a325659c4ab3b4ac7f63c))
* **access-guide:** code-changed to invocation property ([4b1459f](https://github.com/PortakiApp/portaki-modules/commit/4b1459f13b1c62737e5fa8bf25d51ff5f6ab6e17))
* **access-guide:** declare revealed secrets ([8e58c00](https://github.com/PortakiApp/portaki-modules/commit/8e58c00f724c080cdd8325d56d58f0d9d4e37bc1))
* **access-guide:** email only on guest code change ([e4c145f](https://github.com/PortakiApp/portaki-modules/commit/e4c145fbe7176af8441400d7285e3e166841d387))
* **access-guide:** example for onConfigUpdated ([a895085](https://github.com/PortakiApp/portaki-modules/commit/a89508585dd6dd1d61d80dd92ad896c873dc97b8))
* **access-guide:** hide codes after checkout ([3d60972](https://github.com/PortakiApp/portaki-modules/commit/3d609726e5dd06dc8d800463fc09266304167370))
* **access-guide:** hide reveal for no-code methods ([2992adc](https://github.com/PortakiApp/portaki-modules/commit/2992adc9db9d02738c4c153207e363007b794ae9))
* **access-guide:** no legacy panic, https-only links ([6daf6eb](https://github.com/PortakiApp/portaki-modules/commit/6daf6ebf76c5ce1fa2c1e2ff233fa143de7f63f0))
* **access-guide:** skip stay-link emailContext ([2c0dc1c](https://github.com/PortakiApp/portaki-modules/commit/2c0dc1c11d33d729cad2900357fd670942258d99))
* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear Clippy -D warnings on CI ([9728b67](https://github.com/PortakiApp/portaki-modules/commit/9728b67d829b9b31ffee4534051cd1f64e855b5f))
* **modules:** clear clippy dead-code and lifetime lint ([b382ddd](https://github.com/PortakiApp/portaki-modules/commit/b382ddd979d1b3abf1d6f951a19a88417f52dc65))
* **modules:** rattraper le champ booking_channel du SDK ([c74e36c](https://github.com/PortakiApp/portaki-modules/commit/c74e36c417156ebc742e8bc72dc0ec22c3883219))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** sync module.json versions for publish ([647edd7](https://github.com/PortakiApp/portaki-modules/commit/647edd7deda39a896828222571d88fd8ac98ed55))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
* **wifi-guest:** declare revealed secrets ([5a89c0a](https://github.com/PortakiApp/portaki-modules/commit/5a89c0a6122970e563c94631ec6f6dd4c0273a1d))
</details>

<details><summary>appliances: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/appliances-v1.0.0...appliances-v1.1.0) (2026-09-29)


### Features

* **appliances:** add catalogue listing ([cccfee9](https://github.com/PortakiApp/portaki-modules/commit/cccfee989b99454d5b515414aebf4b9f46ae1875))
* **appliances:** dispatch examples, scenario tests ([85955d6](https://github.com/PortakiApp/portaki-modules/commit/85955d6595fb9c6abbe89467611319eda6efa973))
* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** open cards via fullscreen overlay ([530aa1e](https://github.com/PortakiApp/portaki-modules/commit/530aa1e3670d88e906255199bcf736613b9e28c8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **appliances:** examples for delete and reorder ([34962ed](https://github.com/PortakiApp/portaki-modules/commit/34962ede51f42eb7406273142d4fc1256e5847b6))
* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear Clippy -D warnings on CI ([9728b67](https://github.com/PortakiApp/portaki-modules/commit/9728b67d829b9b31ffee4534051cd1f64e855b5f))
* **modules:** clear clippy dead-code and lifetime lint ([b382ddd](https://github.com/PortakiApp/portaki-modules/commit/b382ddd979d1b3abf1d6f951a19a88417f52dc65))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>checklist: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/checklist-v1.0.0...checklist-v1.1.0) (2026-09-29)


### Features

* **checklist:** block publish until a list has items ([da76ee7](https://github.com/PortakiApp/portaki-modules/commit/da76ee7ae270c3070ffa1e959b95049456a00898))
* **checklist:** dispatch examples, scenario tests ([8b60c84](https://github.com/PortakiApp/portaki-modules/commit/8b60c84ab31be318057bdc7d5cc56b60107bb0ee))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **checklist:** examples for list, task commands ([41b0c1f](https://github.com/PortakiApp/portaki-modules/commit/41b0c1f1960ca12347efb907971232649373f96e))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>consumables: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v1.0.0...consumables-v1.1.0) (2026-09-29)


### Features

* **consumables:** block publish on empty catalog ([e0f435e](https://github.com/PortakiApp/portaki-modules/commit/e0f435ebfb95c32c9a89c9ddd1fcbfc201e4a9e4))
* **consumables:** dispatch examples, scenario tests ([4fb36a4](https://github.com/PortakiApp/portaki-modules/commit/4fb36a44e9c5ef3dab3c12b82c8ee627f179446b))
* **consumables:** show item and level as rows ([1ea2703](https://github.com/PortakiApp/portaki-modules/commit/1ea270399794efd993317fede1cd5a14b6d35497))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **consumables:** example for updateStatus ([51939d6](https://github.com/PortakiApp/portaki-modules/commit/51939d633a43f522966e94f7dde8eb1bbe31c72f))
* **consumables:** no label key in example input ([fd1daa7](https://github.com/PortakiApp/portaki-modules/commit/fd1daa77903898abe252b7700cf1d0a39979b916))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>emergency-contacts: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/emergency-contacts-v1.0.0...emergency-contacts-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **emergency-contacts:** add catalogue listing ([992d830](https://github.com/PortakiApp/portaki-modules/commit/992d83048a7e51472f2c0231b40e764ffad56ec4))
* **emergency-contacts:** check host phone to publish ([d61cbb4](https://github.com/PortakiApp/portaki-modules/commit/d61cbb44413481cdd51bf75da84ce1dec39cee9e))
* **emergency-contacts:** declare config, phone required ([364fd43](https://github.com/PortakiApp/portaki-modules/commit/364fd436567502aac624bed8e1cf9f3b051fe48c))
* **emergency-contacts:** dispatch examples, scenario tests ([bf52299](https://github.com/PortakiApp/portaki-modules/commit/bf52299bdb29e9a3b377d520d3125174999ac9a8))
* **emergency-contacts:** translate contacts ([454347c](https://github.com/PortakiApp/portaki-modules/commit/454347c6902df4714ee177e3764b9c325c19dfbf))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** emailContext for rules and guides ([d625b5b](https://github.com/PortakiApp/portaki-modules/commit/d625b5b4789a44d63b39435e5fb327c810497c00))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **previews:** render guest surfaces for the catalogue ([73dd3e5](https://github.com/PortakiApp/portaki-modules/commit/73dd3e53c3a649d2c3cb9aa6ac312bfd44d97618))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear clippy dead-code and lifetime lint ([b382ddd](https://github.com/PortakiApp/portaki-modules/commit/b382ddd979d1b3abf1d6f951a19a88417f52dc65))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>ev-parking: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/ev-parking-v1.0.0...ev-parking-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **ev-parking:** add catalogue listing ([e9e45c4](https://github.com/PortakiApp/portaki-modules/commit/e9e45c4905b5996d675951c6003d438c60edd1f5))
* **ev-parking:** add EV spot codes with reveal ([97cab55](https://github.com/PortakiApp/portaki-modules/commit/97cab5593ad9a1e3d40ba84ff0ab669a7df2375b))
* **ev-parking:** declare config, codes as secret ([1bd2459](https://github.com/PortakiApp/portaki-modules/commit/1bd24591146fe9a98af085cdf19b1c4dcaabfc7c))
* **ev-parking:** dispatch examples, scenario tests ([4aa641a](https://github.com/PortakiApp/portaki-modules/commit/4aa641a3eb9602664c5d2e0274f5bfc169d442f6))
* **ev-parking:** translate spot and instructions ([49fa4aa](https://github.com/PortakiApp/portaki-modules/commit/49fa4aa9ef1f61938cdf671e7d300b014f4b9ab3))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **ev-parking:** declare map_url as url, https only ([5aecb67](https://github.com/PortakiApp/portaki-modules/commit/5aecb671047651e9fd7b84ed4a61780d86821658))
* **ev-parking:** declare revealed secrets ([24073a6](https://github.com/PortakiApp/portaki-modules/commit/24073a63a3acb365aba782d0e89d459eb72084ee))
* **ev-parking:** hide codes after checkout ([e8af8ed](https://github.com/PortakiApp/portaki-modules/commit/e8af8ed12a4a4bf942892df6a29e93ee4b006a2c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
* **wifi-guest:** declare revealed secrets ([5a89c0a](https://github.com/PortakiApp/portaki-modules/commit/5a89c0a6122970e563c94631ec6f6dd4c0273a1d))
</details>

<details><summary>events: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/events-v1.0.0...events-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **events:** add catalogue listing ([a0001f5](https://github.com/PortakiApp/portaki-modules/commit/a0001f50010690a678ac74da37cece0d0195e7f3))
* **events:** declare config, manual events recommended ([07d7514](https://github.com/PortakiApp/portaki-modules/commit/07d75149a5287b6183de812748740efc7db9dbc9))
* **events:** dispatch examples, scenario tests ([6431530](https://github.com/PortakiApp/portaki-modules/commit/64315304363f794d2f81c0b55c7e8d8dae3cfd52))
* **events:** expose located events as map markers ([2cbd231](https://github.com/PortakiApp/portaki-modules/commit/2cbd231469a02fa7596ab8e95e7c833b830ffd6c))
* **events:** fetch nearby OpenAgenda events ([35cb74a](https://github.com/PortakiApp/portaki-modules/commit/35cb74a6c06c0a53a7958913b00123df6023a91d))
* **events:** translate events and disclaimer ([924cf2c](https://github.com/PortakiApp/portaki-modules/commit/924cf2c13775fa6d3c469b7ef7ea936265894597))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** add events, nuki, and wifi-guest ([192517e](https://github.com/PortakiApp/portaki-modules/commit/192517e9a071c7c2565bce7c2fc3b09e482bc177))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **consumables,events:** use host clock in render path ([3ce36bf](https://github.com/PortakiApp/portaki-modules/commit/3ce36bfd2e59f186d02400c16502554255ee417e))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **events,local-guide:** one label for all slots ([30d2c84](https://github.com/PortakiApp/portaki-modules/commit/30d2c84a1026659267ce113813bb21e02213a995))
* **events:** guest shell and property coordinates ([c614028](https://github.com/PortakiApp/portaki-modules/commit/c61402878b2da200531e523a32dae0e38db9e26f))
* **events:** upcoming card skips past events ([481cea6](https://github.com/PortakiApp/portaki-modules/commit/481cea63cf67610df82478d45e0d543b6abc5f72))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
</details>

<details><summary>facility-hours: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/facility-hours-v1.0.0...facility-hours-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **facility-hours:** add catalogue listing ([a87a406](https://github.com/PortakiApp/portaki-modules/commit/a87a4062338b4ce0ceeab8ad834d8f4ead95e022))
* **facility-hours:** declare config, facilities required ([358fcb1](https://github.com/PortakiApp/portaki-modules/commit/358fcb1288e04accdbe68ec8d76857b08738a484))
* **facility-hours:** edit lines and note in form ([b067ece](https://github.com/PortakiApp/portaki-modules/commit/b067ece6f5d577b93e4ad836674cac0d18be0acf))
* **facility-hours:** translate facilities ([d7d0c29](https://github.com/PortakiApp/portaki-modules/commit/d7d0c294a0ddbebc3cea068b705e4950baac44dd))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>guest-reviews: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/guest-reviews-v1.0.0...guest-reviews-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest-reviews:** add catalogue listing ([7cd9b06](https://github.com/PortakiApp/portaki-modules/commit/7cd9b06cc211e4f9515b58e3aa88d7c7f228a4b3))
* **guest-reviews:** add stats tile and detail ([67d2a92](https://github.com/PortakiApp/portaki-modules/commit/67d2a92a170292093fa98b2eddb71785c220d899))
* **guest-reviews:** declare config ([7c76249](https://github.com/PortakiApp/portaki-modules/commit/7c76249046589db90f31ee4cab6e0f7d14355d9e))
* **guest-reviews:** describe host toggles ([80bffd4](https://github.com/PortakiApp/portaki-modules/commit/80bffd49853c3b5b0a730c547e357057eba0a90f))
* **guest-reviews:** dispatch examples, scenario tests ([b73c93b](https://github.com/PortakiApp/portaki-modules/commit/b73c93b2d24a72a10155ca1c3a3b72745d7be5de))
* **guest-reviews:** multi-select feasible platforms ([3274eb4](https://github.com/PortakiApp/portaki-modules/commit/3274eb470a833f31fc032cc6186da141731d48da))
* **guest-reviews:** open guest operations explicitly ([fa5a509](https://github.com/PortakiApp/portaki-modules/commit/fa5a509e593331f916e2554df183c3d9cb3f4004))
* **guest-reviews:** platform-aware channel selection (auto mode) ([c7d3eb9](https://github.com/PortakiApp/portaki-modules/commit/c7d3eb99cbcd809df8d10d8d9cec1a311bee48e2))
* **guest-reviews:** require a review platform ([2a3e13e](https://github.com/PortakiApp/portaki-modules/commit/2a3e13e5fd9291f84b8cbd6f848cf7f1c3952637))
* **guest-reviews:** show guest and rating as rows ([2a488f9](https://github.com/PortakiApp/portaki-modules/commit/2a488f9182376a2d95ce5c6791528d8a389a6360))
* **guest-reviews:** show response rate on stats detail ([25d70f0](https://github.com/PortakiApp/portaki-modules/commit/25d70f09a580d5cdb988fb56f9b755ec0f72c9bc))
* **guest-reviews:** translate thank-you message ([bee5ba6](https://github.com/PortakiApp/portaki-modules/commit/bee5ba64cbc9d448b08c8a6a8e6924bb89ca9e4a))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** emit code-changed and review submitted ([7657a9f](https://github.com/PortakiApp/portaki-modules/commit/7657a9fb036ee218698be08f5bd4486cd2382d0f))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** own mail and lost-found host SDUI ([a62e5be](https://github.com/PortakiApp/portaki-modules/commit/a62e5be10fa3049690cdb6185975d7175933e83e))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **guest-reviews:** enable in-booklet Portaki reviews by default ([1730ea1](https://github.com/PortakiApp/portaki-modules/commit/1730ea19982e0292edb4ed4a05000bb3e5296017))
* **guest-reviews:** key reviews under stay prefix ([91a1021](https://github.com/PortakiApp/portaki-modules/commit/91a1021fba46886c18e0166045f84656c1a22bdc))
* **guest-reviews:** no review before arrival ([3a17746](https://github.com/PortakiApp/portaki-modules/commit/3a17746fb19e5eaa2606f405037671572cc74445))
* **guest-reviews:** one review per stay ([85900c4](https://github.com/PortakiApp/portaki-modules/commit/85900c4cded967295162d751e2a4323ff05bd9bd))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>ical-sync: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/ical-sync-v1.0.0...ical-sync-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **ical-sync:** add catalogue listing ([220dc47](https://github.com/PortakiApp/portaki-modules/commit/220dc47feb34916abc0b555b1ad49b0b9a223eb3))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **ical-sync:** add host sync emails via Wasm ([6fdcb49](https://github.com/PortakiApp/portaki-modules/commit/6fdcb499e0a344d9631daca1f908fe4957194d3e))
* **ical-sync:** chart sync history and stays ([4984cbd](https://github.com/PortakiApp/portaki-modules/commit/4984cbd0648377a49d353f4d6b9b9bf92a2f94c7))
* **ical-sync:** count arrivals over stats period ([766ca0e](https://github.com/PortakiApp/portaki-modules/commit/766ca0e872fc25c8cd85df8ce8c3d67095f91af6))
* **ical-sync:** declare calendars config ([427d536](https://github.com/PortakiApp/portaki-modules/commit/427d536fe8c9de0dac18d5eefd40784bea611ec2))
* **ical-sync:** declare per-feed ICS format ([a14f67f](https://github.com/PortakiApp/portaki-modules/commit/a14f67fd65ff6bba8c2f4103811a8f44b38981b4))
* **ical-sync:** detect booking channel per stay row ([7609290](https://github.com/PortakiApp/portaki-modules/commit/76092900b9179dfa35b209827aab7add905d0832))
* **ical-sync:** dispatch examples, scenario tests ([9577830](https://github.com/PortakiApp/portaki-modules/commit/9577830c512f624de4a7f6e607a4bc960951bae1))
* **ical-sync:** merge format + platform into one selector ([a678eef](https://github.com/PortakiApp/portaki-modules/commit/a678eef70b06a84f2d811258dd5770096d45c05c))
* **ical-sync:** multi-calendar sync and stats card ([ec339e1](https://github.com/PortakiApp/portaki-modules/commit/ec339e1ea24018622e32790e7f0f10fec62e211b))
* **ical-sync:** persist calendars[] only ([67d54b1](https://github.com/PortakiApp/portaki-modules/commit/67d54b11d2462157aae6b48839950c554fa3b479))
* **ical-sync:** render sync emails with blocks ([dc0c96a](https://github.com/PortakiApp/portaki-modules/commit/dc0c96a40246e762dfac4ea3ef2aef01ef1f9087))
* **ical-sync:** serve stats tile ([6fdb552](https://github.com/PortakiApp/portaki-modules/commit/6fdb552253e3520cb474ccd4fcc3536c445ab2b2))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **ical-sync:** derive Default for ModuleConfig ([d7b073f](https://github.com/PortakiApp/portaki-modules/commit/d7b073f4bd59f31478f216431e9ddfe7e72f84af))
* **ical-sync:** require the scheduled sync capability ([9690031](https://github.com/PortakiApp/portaki-modules/commit/9690031c237f80cd5d8e34d54a64216abd082df7))
* **ical-sync:** send one email for all failed feeds ([e733358](https://github.com/PortakiApp/portaki-modules/commit/e73335859b68d2d0017adf53160addf11b881946))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>issue-report: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.0.0...issue-report-v1.1.0) (2026-09-29)


### Features

* **issue-report:** dispatch examples, scenario tests ([4c4ea9b](https://github.com/PortakiApp/portaki-modules/commit/4c4ea9b81c67a6396b07bf39d5bcc1fa2143ea16))
* **issue-report:** show category and photo as blocks ([6716217](https://github.com/PortakiApp/portaki-modules/commit/6716217d19e1878bc7d7e6284443a9eaf57a62a3))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **issue-report:** example for resolve ([0eda02e](https://github.com/PortakiApp/portaki-modules/commit/0eda02ea6ddc2aa9854d57a9aecc7bfaa9864c11))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>local-guide: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/local-guide-v1.0.0...local-guide-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **events:** expose located events as map markers ([2cbd231](https://github.com/PortakiApp/portaki-modules/commit/2cbd231469a02fa7596ab8e95e7c833b830ffd6c))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **local-guide:** accept a pasted destination URL ([87c389b](https://github.com/PortakiApp/portaki-modules/commit/87c389bac29072910e050e3a780f592cf0016023))
* **local-guide:** add catalogue listing ([3fa1450](https://github.com/PortakiApp/portaki-modules/commit/3fa1450e4b4f55451c972317205a50c11faccfcd))
* **local-guide:** declare config, places required ([9c8e3c5](https://github.com/PortakiApp/portaki-modules/commit/9c8e3c5bff30cce13f95b199fb31520c4151cc2a))
* **local-guide:** dispatch examples, scenario tests ([8a26594](https://github.com/PortakiApp/portaki-modules/commit/8a26594ae29b76d8fb4f1902842a48cae4867189))
* **local-guide:** expose spots as map markers ([841c311](https://github.com/PortakiApp/portaki-modules/commit/841c31173b89c682b3d433ec2484e1ea617f307b))
* **local-guide:** list nearby Tiqets tickets ([0e69f1f](https://github.com/PortakiApp/portaki-modules/commit/0e69f1f6da07033f47a8241fb0ac7eab6f4a24f2))
* **local-guide:** map the host's located spots ([05a4874](https://github.com/PortakiApp/portaki-modules/commit/05a48742d04ebdef193a9ba4e04b951f29da1065))
* **local-guide:** offer GetYourGuide activities as partner links ([51b0676](https://github.com/PortakiApp/portaki-modules/commit/51b06769c2c22e32b4f73601ccf72a1590464eb7))
* **local-guide:** set the GetYourGuide partner id ([132d60b](https://github.com/PortakiApp/portaki-modules/commit/132d60b13b5492eb8e63bd2940c7f63691cf8df3))
* **local-guide:** translate spots and activities ([3c0acbd](https://github.com/PortakiApp/portaki-modules/commit/3c0acbd2396750d5b214a24756abada379a68e04))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** emailContext for rules and guides ([d625b5b](https://github.com/PortakiApp/portaki-modules/commit/d625b5b4789a44d63b39435e5fb327c810497c00))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **previews:** render guest surfaces for the catalogue ([73dd3e5](https://github.com/PortakiApp/portaki-modules/commit/73dd3e53c3a649d2c3cb9aa6ac312bfd44d97618))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **events,local-guide:** one label for all slots ([30d2c84](https://github.com/PortakiApp/portaki-modules/commit/30d2c84a1026659267ce113813bb21e02213a995))
* **local-guide:** guest shell and property coordinates ([db46eb6](https://github.com/PortakiApp/portaki-modules/commit/db46eb65b1fc01f2c4c2df0bfe3db99ce78b37d0))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>lost-found: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.0.0...lost-found-v1.1.0) (2026-09-29)


### Features

* **lost-found:** dispatch examples, scenario tests ([05868f2](https://github.com/PortakiApp/portaki-modules/commit/05868f2dfe86f46aaf80cdf6d399c30ae10b7791))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **lost-found:** declare host-found guest email ([a143bdb](https://github.com/PortakiApp/portaki-modules/commit/a143bdb2be2a2c6793f4de553f66281f37427cb5))
* **lost-found:** example for updateStatus ([3bb8f9a](https://github.com/PortakiApp/portaki-modules/commit/3bb8f9af804310ee43870c048e71d4fbbfbf9bf6))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>nuki: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/nuki-v1.0.0...nuki-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** add events, nuki, and wifi-guest ([192517e](https://github.com/PortakiApp/portaki-modules/commit/192517e9a071c7c2565bce7c2fc3b09e482bc177))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **nuki:** add catalogue listing ([bc34bf1](https://github.com/PortakiApp/portaki-modules/commit/bc34bf1f455c65ccda81828d30b810b38c83dcfa))
* **nuki:** declare config, keypad code as secret ([48dac5d](https://github.com/PortakiApp/portaki-modules/commit/48dac5d688f08b8b4e2905439a5766a1496e5366))
* **nuki:** declare it feeds access-guide ([ac80359](https://github.com/PortakiApp/portaki-modules/commit/ac80359ddabe8efba612d9e180edac5d42eb89c3))
* **nuki:** dispatch examples, scenario tests ([f12cc74](https://github.com/PortakiApp/portaki-modules/commit/f12cc741ab353d5d5c45f581567242ccf47aa964))
* **nuki:** open guest operations explicitly ([fdbfc8b](https://github.com/PortakiApp/portaki-modules/commit/fdbfc8b2a96c16af14a030b1e6fb036ad6445d76))
* **nuki:** remote unlock via BYOK connector ([b300c6a](https://github.com/PortakiApp/portaki-modules/commit/b300c6ab06bd66e26228298b0888e29c38dd8261))


### Bug Fixes

* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **nuki:** refuse lock outside the stay window ([b4a8fb7](https://github.com/PortakiApp/portaki-modules/commit/b4a8fb7d90edb3b40860dfec1d2442d989d52c57))
</details>

<details><summary>pre-arrival-form: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v1.0.0...pre-arrival-form-v1.1.0) (2026-09-29)


### Features

* **pre-arrival-form:** declare config ([40ad78a](https://github.com/PortakiApp/portaki-modules/commit/40ad78a5603afbfc967f6c3f8ae2f6dc8f851c7e))
* **pre-arrival-form:** dispatch examples, scenario tests ([2484708](https://github.com/PortakiApp/portaki-modules/commit/24847088fd7365bb3a4187c5fc0d30df6ffd4f00))
* **pre-arrival-form:** preview asked fields in email ([afe8447](https://github.com/PortakiApp/portaki-modules/commit/afe8447f3af97ba30c27d2b548fb46f9055f8d9f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **pre-arrival-form:** keep id document out of event ([f9428a1](https://github.com/PortakiApp/portaki-modules/commit/f9428a15f2d4ddde5e76dfd0cbecbe642a652f57))
* **pre-arrival-form:** no form invite after check-in ([469f706](https://github.com/PortakiApp/portaki-modules/commit/469f706823c21d141e7255952c91c2d048871717))
</details>

<details><summary>rules: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/rules-v1.0.0...rules-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** open cards via fullscreen overlay ([530aa1e](https://github.com/PortakiApp/portaki-modules/commit/530aa1e3670d88e906255199bcf736613b9e28c8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** emailContext for rules and guides ([d625b5b](https://github.com/PortakiApp/portaki-modules/commit/d625b5b4789a44d63b39435e5fb327c810497c00))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **rules:** add catalogue listing ([d05ce05](https://github.com/PortakiApp/portaki-modules/commit/d05ce05fbb769187c37966cbda2cd5c128fe814f))
* **rules:** add updateConfig for workspace Save ([6edce79](https://github.com/PortakiApp/portaki-modules/commit/6edce79ad53e34ad307a9ffb7b4e67e11521036f))
* **rules:** align Règlement intérieur SDUI ([74c4887](https://github.com/PortakiApp/portaki-modules/commit/74c4887edca08281022aab49956fad6e2ada6084))
* **rules:** dispatch examples, scenario tests ([5adb5a5](https://github.com/PortakiApp/portaki-modules/commit/5adb5a55d2b72e0674e307cb4a636ac049a8e133))
* **rules:** require a rule before publish ([d086b5b](https://github.com/PortakiApp/portaki-modules/commit/d086b5b97894758cb07d3e08d24c81a075fb010f))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear clippy dead-code and lifetime lint ([b382ddd](https://github.com/PortakiApp/portaki-modules/commit/b382ddd979d1b3abf1d6f951a19a88417f52dc65))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
</details>

<details><summary>sections: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/sections-v1.0.0...sections-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **sections:** add catalogue listing ([aab8633](https://github.com/PortakiApp/portaki-modules/commit/aab8633ee6fc498a4a271bab7e0ed77d3adb228b))
* **sections:** align host SDUI to design ([0e7e769](https://github.com/PortakiApp/portaki-modules/commit/0e7e769f5580ee2611848ca9c4cc4d1629704677))
* **sections:** align welcome module with the design ([4934945](https://github.com/PortakiApp/portaki-modules/commit/4934945f6488991ba54003d4b7dc39039c40eb3d))
* **sections:** dispatch examples, scenario tests ([8ca345a](https://github.com/PortakiApp/portaki-modules/commit/8ca345aa9afa560a0f3daac0bd3010414cc58e51))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **sections:** examples for delete and reorder ([52e05b7](https://github.com/PortakiApp/portaki-modules/commit/52e05b747d612f46d807e53765560210a25954c2))
</details>

<details><summary>train: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.0.0...train-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** open cards via fullscreen overlay ([530aa1e](https://github.com/PortakiApp/portaki-modules/commit/530aa1e3670d88e906255199bcf736613b9e28c8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear Clippy -D warnings on CI ([9728b67](https://github.com/PortakiApp/portaki-modules/commit/9728b67d829b9b31ffee4534051cd1f64e855b5f))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
</details>

<details><summary>waste-recycling: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/waste-recycling-v1.0.0...waste-recycling-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **waste-recycling:** add catalogue listing ([2baf761](https://github.com/PortakiApp/portaki-modules/commit/2baf76108d64b82ac4e8d4f87c890013bf56cecf))
* **waste-recycling:** declare config, bins required ([36478e0](https://github.com/PortakiApp/portaki-modules/commit/36478e0abe2d99ed05e61ccc0459528c6790b515))
* **waste-recycling:** paint bins with named swatches ([c2884dd](https://github.com/PortakiApp/portaki-modules/commit/c2884ddb5935c12a99fee3d38c8811723cb1f323))
* **waste-recycling:** translate bins ([a1ec00e](https://github.com/PortakiApp/portaki-modules/commit/a1ec00ef66dd2e513d42d185f1ea39d4d1fce20d))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **waste-recycling:** show one item per line hint ([c71e7b6](https://github.com/PortakiApp/portaki-modules/commit/c71e7b60f7c1e9d4ee5463b4de0f0cdbb1850c11))
</details>

<details><summary>weather: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/weather-v1.0.0...weather-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** templated nav-row summaries for weather & checklist ([42ba408](https://github.com/PortakiApp/portaki-modules/commit/42ba40879114280f1a81f9c506cfffa6e2e5748c))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** add emailContext guest queries ([5d8baeb](https://github.com/PortakiApp/portaki-modules/commit/5d8baeb2f5098bb7eb247620273368827e737d1d))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **weather,checklist:** lead the guest now-card with a live title ([d8a3ce0](https://github.com/PortakiApp/portaki-modules/commit/d8a3ce034314d832ab644f69e3074168881c271d))
* **weather:** add catalogue listing ([46c662e](https://github.com/PortakiApp/portaki-modules/commit/46c662e76ab056e139a8a8d47adebc157491e4a4))
* **weather:** declare units and refresh config ([37a8734](https://github.com/PortakiApp/portaki-modules/commit/37a8734f41ff47ae9b827d8192ff9c957e3a5b07))
* **weather:** dispatch examples, scenario tests ([ebf2848](https://github.com/PortakiApp/portaki-modules/commit/ebf2848eafe364d176612563372bc34309a79236))
* **weather:** drive home-card glance from live conditions ([66e2e52](https://github.com/PortakiApp/portaki-modules/commit/66e2e522078d44bc8e2c1289e1d354c549c2c03b))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **i18n:** fill weather keys; harden wifi reveal ([d7cce9c](https://github.com/PortakiApp/portaki-modules/commit/d7cce9c846a1c070a68a475005bcf5d1039087ed))
* **modules:** clear Clippy -D warnings on CI ([9728b67](https://github.com/PortakiApp/portaki-modules/commit/9728b67d829b9b31ffee4534051cd1f64e855b5f))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** sync module.json versions for publish ([647edd7](https://github.com/PortakiApp/portaki-modules/commit/647edd7deda39a896828222571d88fd8ac98ed55))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
* **weather:** hide forecast when not geocoded ([5e68d4c](https://github.com/PortakiApp/portaki-modules/commit/5e68d4cbc5f230bbe43f8dd22eadce5a5636ecca))
* **weather:** return no weather instead of failing ([b06144c](https://github.com/PortakiApp/portaki-modules/commit/b06144cc117850f965b1b948f7b6d7bdd158b9fe))
</details>

<details><summary>wifi-guest: 1.1.0</summary>

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v1.0.0...wifi-guest-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** add events, nuki, and wifi-guest ([192517e](https://github.com/PortakiApp/portaki-modules/commit/192517e9a071c7c2565bce7c2fc3b09e482bc177))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **wifi-guest:** add catalogue listing ([251f1a9](https://github.com/PortakiApp/portaki-modules/commit/251f1a9f3f54471d8df8cc7ac89504eb8e06f018))
* **wifi-guest:** check network before publish ([5f9e8f1](https://github.com/PortakiApp/portaki-modules/commit/5f9e8f1cdd0dd9d0c9113bbff7e47726bb2ff7d7))
* **wifi-guest:** declare config, password as secret ([dfe3b61](https://github.com/PortakiApp/portaki-modules/commit/dfe3b61c96271ee257e192571790d79a1228e078))
* **wifi-guest:** dispatch examples, scenario tests ([4893ed3](https://github.com/PortakiApp/portaki-modules/commit/4893ed3a7f5a4ec553fb97bf282e09075df6e14a))
* **wifi-guest:** translate hint and steps ([ce481b1](https://github.com/PortakiApp/portaki-modules/commit/ce481b10950164c8dd0f1ebfc8fb4882d611f548))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **i18n:** fill weather keys; harden wifi reveal ([d7cce9c](https://github.com/PortakiApp/portaki-modules/commit/d7cce9c846a1c070a68a475005bcf5d1039087ed))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
* **wifi-guest:** declare revealed secrets ([5a89c0a](https://github.com/PortakiApp/portaki-modules/commit/5a89c0a6122970e563c94631ec6f6dd4c0273a1d))
* **wifi-guest:** flatten host drawer SDUI form ([cfcd54d](https://github.com/PortakiApp/portaki-modules/commit/cfcd54de230125dee5b4884e864441235049c8bb))
* **wifi-guest:** hide password after checkout ([1b048ac](https://github.com/PortakiApp/portaki-modules/commit/1b048ac41286bcae3e4f3c41cde1bfbc745b9a42))
</details>

---
## Before merge

- [ ] CHANGELOG entries look correct per module
- [ ] An SDK bump is NOT in here. release-please attributes commits by path (`No commits for path: modules/...`), so a root-only bump releases nothing and the artifacts keep announcing the old SDK -- see the root `Cargo.toml`
- [ ] After merge, `ci` publish on `main` should publish to Portaki from bumped Cargo.toml versions

Config is generated by `scripts/generate-release-please-config.sh` -- do not hand-edit package paths.