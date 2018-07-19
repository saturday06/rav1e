bench: target/release/rav1e input.y4m aom_test/aomdec
	bash -c "time target/release/rav1e input.y4m --output output_s10_q255.ivf --speed 10 --quantizer 255 2> output_s10_q255.ivf.txt"
	bash -c "time target/release/rav1e input.y4m --output output_s10_q200.ivf --speed 10 --quantizer 200 2> output_s10_q200.ivf.txt"
	bash -c "time target/release/rav1e input.y4m --output output_s10.ivf --speed 10 2> output_s10.ivf.txt"
	bash -c "time target/release/rav1e input.y4m --output output.ivf 2> output.ivf.txt"
	bash -c "time target/release/rav1e input.y4m --output output_s10_q0.ivf   --speed 10 --quantizer 0   2> output_s10_q0.ivf.txt"
	./aom_test/aomdec output_s10_q255.ivf -o output_s10_q255.y4m
	./aom_test/aomdec output_s10_q200.ivf -o output_s10_q200.y4m
	./aom_test/aomdec output_s10.ivf -o output_s10.y4m
	./aom_test/aomdec output.ivf -o output.y4m
	./aom_test/aomdec output_s10_q0.ivf -o output_s10_q0.y4m

aom_test/aomdec:
	mkdir aom_test
	(cd aom_test \
		&& cmake ../aom_build/aom \
			-DAOM_TARGET_CPU=generic \
			-DCONFIG_AV1_ENCODER=0 \
			-DCONFIG_UNIT_TESTS=0 \
			-DENABLE_DOCS=0 \
			-DCONFIG_EXT_PARTITION_TYPES=0 \
			-DCONFIG_INTRA_EDGE2=0 \
			-DCONFIG_OBU=1 \
			-DCONFIG_FILTER_INTRA=1 \
			-DCONFIG_MONO_VIDEO=1 \
			-DCONFIG_Q_ADAPT_PROBS=1 \
			-DCONFIG_SCALABILITY=1 \
			-DCONFIG_OBU_SIZING=1 \
			-DCONFIG_TIMING_INFO_IN_SEQ_HEADERS=0 \
			-DCONFIG_FILM_GRAIN=0 \
		&& make -j8)

input.y4m:
	curl -Lf http://samples.mplayerhq.hu/yuv4mpeg2/example.y4m.bz2 -o input.y4m.bz2
	bzip2 -d input.y4m.bz2

target/release/rav1e:
	cargo build --release
