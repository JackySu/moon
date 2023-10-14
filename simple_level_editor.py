from dataclasses import dataclass
from enum import Enum, unique
import pygame
import os


ENCODING = 'UTF-8'

PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD = 5.

LINE_COLOR = (255, 255, 255)
LINE_WIDTH = 6
STAR_COLOR = (255, 255, 0)
STAR_RADIUS = 10
PLAYER_COLOR = (255, 0, 0)
PLAYER_RADIUS = 15

WIDTH = 1280
HEIGHT = 720

HALF_WIDTH = WIDTH // 2
HALF_HEIGHT = HEIGHT // 2


@dataclass
@unique
class DataType(Enum):
    PLAYER = 'p '
    LINE = 'l '
    STAR = 's '


@dataclass
class Star:
    x: float
    y: float

    def cvt_coord(self) -> str:
        return f"{self.x - HALF_WIDTH:.1f},{HALF_HEIGHT - self.y:.1f}"

    def __str__(self):
        return DataType.STAR.value + self.cvt_coord()


@dataclass
class Vertex:
    x: float
    y: float

    def cvt_coord(self) -> str:
        return f"{self.x - HALF_WIDTH:.1f},{HALF_HEIGHT - self.y:.1f}"

    def __str__(self):
        return self.cvt_coord()


@dataclass
class PolyLine:
    vertices: list[Vertex]

    def __str__(self):
        return DataType.LINE.value + ' '.join([str(vertex) for vertex in self.vertices])


@dataclass
class Player:
    x: float
    y: float

    def cvt_coord(self) -> str:
        return f"{self.x - HALF_WIDTH:.1f},{HALF_HEIGHT - self.y:.1f}"

    def __str__(self):
        return DataType.PLAYER.value + self.cvt_coord()


def anti_cvt_coord(x: float, y: float) -> tuple[float, float]:
    return (x + HALF_WIDTH, HALF_HEIGHT - y)


def read_stars_from_line(line: str) -> list[Star]:
    stars = []
    if line[:2] != DataType.STAR.value or len(line) <= 2:
        return stars

    for star in line.split()[1:]:
        x, y = star.split(',')
        sx, sy = anti_cvt_coord(float(x), float(y))
        stars.append(Star(sx, sy))
    return stars


def read_vertices_from_line(line: str) -> list[Vertex]:
    vertices = []
    if line[:2] != DataType.LINE.value or len(line) <= 2:
        return vertices

    for vertex in line.split()[1:]:
        x, y = vertex.split(',')
        vx, vy = anti_cvt_coord(float(x), float(y))
        vertices.append(Vertex(vx, vy))
    return vertices


def parse_from_file(filepath: str) -> tuple[list[PolyLine], list[Star], Player]:
    lines = []
    stars = []
    player = None
    is_player_read = False
    file_exists = os.path.isfile(filepath)
    try:
        with open(filepath, "rb") as file:
            for line in file.read().decode(encoding=ENCODING).split('\n'):
                if not line or line[0] == '#' or line[0] == '\n' or len(line) <= 2:
                    continue

                if line[:2] == DataType.LINE.value:
                    vertices = read_vertices_from_line(line)
                    if vertices:
                        lines.append(PolyLine(vertices))

                elif line[:2] == DataType.STAR.value:
                    stars.extend(read_stars_from_line(line))

                elif line[:2] == DataType.PLAYER.value:
                    if is_player_read:
                        print("<!> Multiple player start points found, ignored")
                        continue
                    coords = line[2:].split(',')
                    x, y = anti_cvt_coord(float(coords[0]), float(coords[1]))
                    player = Player(x, y)
                    is_player_read = True

    except FileNotFoundError:
        print(f"<!> File {filepath} not found, creating a new one")
        with open(filepath, "wb") as file:
            file.write(b"")

    finally:
        return lines, stars, player, file_exists


def write_to_file(filepath: str, lines: list[PolyLine], stars: list[Star], player: Player):
    with open(filepath, "wb") as file:
        for line in lines:
            file.write((str(line) + '\n').encode(encoding=ENCODING))
        for star in stars:
            file.write((str(star) + '\n').encode(encoding=ENCODING))
        if player:
            file.write((str(player) + '\n').encode(encoding=ENCODING))


def draw(lines: list[PolyLine], stars: list[Star], player: Player):
    for line in lines:
        for i in range(len(line.vertices) - 1):
            pygame.draw.line(screen, LINE_COLOR, (line.vertices[i].x, line.vertices[i].y), (line.vertices[i + 1].x, line.vertices[i + 1].y), LINE_WIDTH)

    for star in stars:
        pygame.draw.circle(screen, STAR_COLOR, (star.x, star.y), STAR_RADIUS)

    if player:
        pygame.draw.circle(screen, PLAYER_COLOR, (player.x, player.y), PLAYER_RADIUS)


if __name__ == "__main__":
    print('''Simple Level Editor for browser game moon
    Controls:
        Left mouse button: Draw a line
        Middle mouse button: Draw player start point
        Right mouse button: Draw a star
        Z: Undo the last line drawn
        X: Undo the last star drawn
        Esc: Exit the program
    ''')
    filename = input("Enter the name of the level: ")
    filepath = f"./src/levels/{filename}.txt"

    pygame.init()
    screen = pygame.display.set_mode((WIDTH, HEIGHT))
    pygame.display.set_caption("Simple Level Editor")

    saved_lines, saved_stars, player, file_exists = parse_from_file(filepath)
                
    appending_lines = []
    appending_stars = []
    # not appending player because it is unique
    current_vertices = []
    current_polyline = None
    drawing = False
    
    TO_REWIND = []

    while playing := True:
        screen.fill((0, 0, 0))
        draw(saved_lines, saved_stars, player)
        if current_polyline:
            draw(appending_lines + [current_polyline], appending_stars, player)
        else:
            draw(appending_lines, appending_stars, player)

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                playing = False
                break
            
            elif event.type == pygame.MOUSEBUTTONDOWN:
                # Left mouse button
                if event.button == 1:
                    drawing = True
                    current_vertices = [Vertex(event.pos[0], event.pos[1])]
                    current_polyline = PolyLine(current_vertices)

                # Middle mouse button
                if event.button == 2:
                    player = Player(event.pos[0], event.pos[1])

                # right mouse button
                elif event.button == 3:
                    appending_stars.append(Star(event.pos[0], event.pos[1]))

            elif event.type == pygame.MOUSEMOTION and drawing:
                if ((current_vertices[-1].x - event.pos[0]) ** 2 + (current_vertices[-1].y - event.pos[1]) ** 2) ** 0.5 > PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD:
                    current_vertices.append(Vertex(event.pos[0], event.pos[1]))
                    current_polyline = PolyLine(current_vertices)

            elif event.type == pygame.MOUSEBUTTONUP:
                if event.button == 1:
                    drawing = False
                    appending_lines.append(current_polyline)

            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_z:
                    if appending_lines:
                        appending_lines.pop()
                        current_polyline = None
                if event.key == pygame.K_x:
                    if appending_stars:
                        appending_stars.pop()

        if not playing:
            break

        pygame.display.flip()

    pygame.quit()

    write_to_file(filepath, saved_lines + appending_lines, saved_stars + appending_stars, player)
