OBJECTS = src/*
all:
	mkdir -p build
	g++ $(OBJECTS) -o build/out -lglfw -lGL -lX11 -lpthread -lXrandr -lXi -ldl -Iinclude

run: all
	build/out

clean:
	rm -rf build