# Technically only support x86_64 for now...
ARCH ?= x86_64
ASSEMBLY := arch/$(ARCH)/boot/*.o
KERNEL := build/$(ARCH)/kernel-$(ARCH).bin
ISOFILES := build/$(ARCH)/isofiles
ISO := build/$(ARCH)/os-$(ARCH).iso
TARGET ?= $(ARCH)-rustbelt
RUSTBELT := target/$(TARGET)/debug/librustbelt.a

.PHONY: $(ASSEMBLY) $(KERNEL) kernel os.iso run

build:
	mkdir -p $(ISOFILES)/boot/grub
	cp etc/grub.cfg $(ISOFILES)/boot/grub

# Compile assembly files
$(ASSEMBLY): arch/$(ARCH)/boot/*.asm
	for f in $?; do nasm -f elf64 $$f; done

$(KERNEL): $(ASSEMBLY)
	$(ARCH)-elf-ld -n --gc-sections -o $(KERNEL) -T arch/$(ARCH)/boot/linker.ld $? $(RUSTBELT)

kernel: 
	@xargo build --target=$(TARGET)

os.iso: kernel $(KERNEL)
	mkdir -p $(ISOFILES)/boot
	mv $(KERNEL) $(ISOFILES)/boot/kernel.bin
	grub-mkrescue -o $(ISO) $(ISOFILES)

run: os.iso
	qemu-system-$(ARCH) -cdrom $(ISO)
