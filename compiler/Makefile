ARCHITECTURES := x86_64 aarch64
OS := unknown-linux-gnu apple-darwin

all := $(foreach arch,$(ARCHITECTURES), \
		$(foreach os,$(OS), \
			$(arch)-$(os) \
		) \
	)

.PHONY: $(all)

$(all):
	cross build --target $@ --release
	cp target/$@/release/ruu builds/ruu.$@
	cp target/$@/release/ruulang-server builds/ruulang-server.$@

clean:
	rm -rf builds/ruu.*
	rm -rf builds/ruulang-server.*

local: x86_64-apple-darwin aarch64-apple-darwin

all: $(all)