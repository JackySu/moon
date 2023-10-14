import pygame
import os


ENCODING = 'UTF-8'

PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD = 5.

LINE_COLOR = (255, 255, 255)
LINE_WIDTH = 6

WIDTH = 1280
HEIGHT = 720

HALF_WIDTH = WIDTH // 2
HALF_HEIGHT = HEIGHT // 2


def anti_cvt_coord(x: float, y: float) -> tuple[float, float]:
    return (x + HALF_WIDTH, HALF_HEIGHT - y)


def draw_saved_level(level: list[str]):
    screen.fill((0, 0, 0))
    for line in level:
        if line[:2] == 'l ':
            vertices = line[2:].split()
            start_point = anti_cvt_coord(float(vertices[0].split(',')[0]), float(vertices[0].split(',')[1]))
            for vertex in vertices[1:]:
                end_point = anti_cvt_coord(float(vertex.split(',')[0]), float(vertex.split(',')[1]))
                pygame.draw.line(screen, LINE_COLOR, start_point, end_point, LINE_WIDTH)
                start_point = end_point

        elif line[:2] == 's ':
            coords = line[2:].split()
            x, y = anti_cvt_coord(float(coords[0].split(',')[0]), float(coords[0].split(',')[1]))
            pygame.draw.circle(screen, LINE_COLOR, x, y, 20)


def redraw_screen():
    screen.fill((0, 0, 0))
    for line in level:
        if line[:2] == 'l ':
            vertices = line[2:].split()
            start_point = cvt_coord(float(vertices[0].split(',')[0]), float(vertices[0].split(',')[1]), False)
            for vertex in vertices[1:]:
                end_point = cvt_coord(float(vertex.split(',')[0]), float(vertex.split(',')[1]), False)
                pygame.draw.line(screen, LINE_COLOR, start_point, end_point, LINE_WIDTH)
                start_point = end_point

        elif line[:2] == 's ':
            coords = line[2:].split()
            x, y = cvt_coord(float(coords[0].split(',')[0]), float(coords[0].split(',')[1]), False)
            pygame.draw.circle(screen, LINE_COLOR, x, y, 20)


def cvt_coord(x: float, y: float, is_str: bool = True) -> str | tuple[float, float]:
    if is_str:
        return f"{x - HALF_WIDTH:.1f},{HALF_HEIGHT - y:.1f} "
    return (x - HALF_WIDTH, HALF_HEIGHT - y)


if __name__ == "__main__":
    filename = input("Enter the name of the level: ")
    filepath = f"./src/levels/{filename}.txt"

    pygame.init()
    screen = pygame.display.set_mode((WIDTH, HEIGHT))
    pygame.display.set_caption("Simple Level Editor")

    vertices = ''
    drawing = False
    start_point = None
    end_point = None
    level = []

    try:
        with open(f"./src/levels/{filename}.txt", "rb") as file:
            saved_level = file.read().decode(encoding=ENCODING).split('\n')
            print(f"Found existing level file at {filepath}, loading")
            draw_saved_level(saved_level)
    except FileNotFoundError:
        print(f"Creating a new level file at {filepath}")
        # create a new file
        with open(filepath, "wb") as file:
            file.write(b"")
                

    while playing := True:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                playing = False
                break

            elif event.type == pygame.MOUSEBUTTONDOWN:
                if event.button == 1:  # Left mouse button
                    drawing = True
                    start_point = event.pos
                    vertices = 'l '

                elif event.button == 2:
                    end_point = event.pos
                    level.append(f"s {cvt_coord(end_point[0], end_point[1])}")

            elif event.type == pygame.MOUSEMOTION:
                if drawing and ((event.pos[0] - start_point[0]) ** 2 +
                        (event.pos[1] - start_point[1]) ** 2) ** 0.5 > PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD:
                    end_point = event.pos
                    pygame.draw.line(screen, LINE_COLOR, start_point, end_point, LINE_WIDTH)
                    vertices += cvt_coord(end_point[0], end_point[1])
                    start_point = end_point

            elif event.type == pygame.MOUSEBUTTONUP:
                if event.button == 1:
                    level.append(vertices)
                    drawing = False

            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_c:
                    level.pop()
                    redraw_screen()
        
        if not playing:
            break
        pygame.display.flip()

    pygame.quit()

    with open(filepath, "ab+") as file:
        if level and os.stat(filepath).st_size != 0:
            file.write(b"\n")
        level = [line.strip() for line in level]
        content = '\n'.join(level).encode(encoding=ENCODING)
        file.write(content)
